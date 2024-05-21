#[cfg(feature = "ssr")]
use crate::auth;
use crate::components::users::{get_user_map, UserInfo};
use futures::future;
use leptonic::components::select::OptionalSelect;
use leptos::{html::Input, *};
#[cfg(feature = "ssr")]
use rustface::{Detector, ImageData};
use serde::{Deserialize, Serialize};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaPayload {
    data: Vec<(String, Vec<u8>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bbox {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[cfg(feature = "ssr")]
impl Bbox {
    fn rect(r: &rustface::Rectangle) -> Bbox {
        Bbox {
            x: r.x() as u32,
            y: r.y() as u32,
            w: r.width(),
            h: r.height(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Person {
    bounds: Option<Bbox>,
    name: String,
    id: usize,
}

#[server(Faces, "/api", "Cbor")]
pub async fn faces(image_b64: String) -> Result<Vec<Bbox>, ServerFnError> {
    let mut detector = match rustface::create_detector("model.bin") {
        Ok(detector) => detector,
        Err(_) => {
            return Err(ServerFnError::new(
                "Failed to create face detector".to_string(),
            ));
        }
    };

    detector.set_min_face_size(20);
    detector.set_score_thresh(2.0);
    detector.set_pyramid_scale_factor(0.8);
    detector.set_slide_window_step(4, 4);

    let image = decode_image(image_b64).to_luma8();
    let faces = detect_faces(&mut *detector, &image);
    Ok(faces)
}

fn decode_image(encoded_string: String) -> image::DynamicImage {
    let bytes = base64::decode(encoded_string).expect("Failed to decode image");
    image::load_from_memory(&bytes).expect("Failed to load image")
}

#[cfg(feature = "ssr")]
fn detect_faces(detector: &mut dyn Detector, gray: &image::GrayImage) -> Vec<Bbox> {
    let (width, height) = gray.dimensions();
    let image = ImageData::new(gray, width, height);
    let faces = detector.detect(&image);

    faces.iter().map(|fi| Bbox::rect(fi.bbox())).collect()
}

#[server(Upload, "/api", "Cbor")]
pub async fn upload_media_server(
    filename: String,
    encoded_string: String,
    names: Vec<String>,
) -> Result<(), ServerFnError> {
    let user = auth::logged_in().await?;
    use crate::db::ssr::pool;
    use rand::Rng;
    use std::fs;
    use std::path::Path;

    let pool = pool()?;

    if !Path::new("./album").exists() {
        let _ = fs::create_dir_all("./album")?;
    }

    use uuid::Uuid;
    let file_ext = extract_ext(filename).expect_throw("Missing file extension");
    let uuid = Uuid::new_v4().to_string();

    let path = format!("./album/{}.{}", uuid, file_ext);
    let bytes = base64::decode(encoded_string).expect_throw("Failed to decode base64");

    fs::write(&path, bytes).expect_throw("Failed to write file");

    sqlx::query(
        "INSERT INTO files (id, path, uploadDate, createdDate, uploadedBy) 
        VALUES (?, ?, datetime('now', 'localtime'), ?, ?)",
    ) //SELECT date('now', 'localtime');
    .bind(&uuid)
    .bind(path)
    //Randomize data for testing
    .bind(
        format!(
            "{}-{:02}-{:02}",
            rand::thread_rng().gen_range(2010..2023),
            rand::thread_rng().gen_range(1..13),
            rand::thread_rng().gen_range(1..29),
        )
        .to_string(),
    )
    .bind(user.id)
    .execute(&pool)
    .await?;

    // Find userIDs from names
    for name in names {
        let user = match auth::ssr::SqlUser::get_from_username(name.clone(), &pool).await {
            Some(u) => u.id,
            None => {
                let res = sqlx::query("INSERT INTO users (username) VALUES (?)")
                    .bind(name)
                    .execute(&pool)
                    .await?;

                res.last_insert_rowid()
            }
        };

        // Insert name tags
        sqlx::query("INSERT INTO userFile (userID, fileID) VALUES (?, ?)")
            .bind(user)
            .bind(&uuid)
            .execute(&pool)
            .await?;
    }

    Ok(())
}

async fn upload(
    payload: Vec<(String, String, RwSignal<Vec<Person>>)>,
    set_done: WriteSignal<usize>,
    done_count: ReadSignal<usize>,
) -> Result<(), ServerFnError> {
    if payload.is_empty() {
        return Err(ServerFnError::new("No files to upload".to_string()));
    }

    let mut calls = Vec::new();

    for (filename, encoded_string, names) in payload {
        calls.push(upload_wrapper(
            filename,
            encoded_string,
            names
                .get_untracked()
                .iter()
                .map(|nf| nf.name.clone())
                .collect(),
            set_done,
            done_count,
        ));
    }

    let results = future::join_all(calls).await;
    for result in results {
        if let Err(e) = result {
            return Err(e);
        }
    }

    Ok(())
}

async fn upload_wrapper(
    filename: String,
    encoded_string: String,
    names: Vec<String>,
    set_done: WriteSignal<usize>,
    done_count: ReadSignal<usize>,
) -> Result<(), ServerFnError> {
    return match upload_media_server(filename, encoded_string, names).await {
        Ok(_) => {
            set_done(done_count.get_untracked() + 1);
            logging::log!("{}", done_count.get_untracked());
            Ok(())
        }
        Err(e) => Err(e),
    };
}

#[component]
pub fn UploadMedia() -> impl IntoView {
    use wasm_bindgen::JsCast;

    let (media, set_media) = create_signal(Vec::new());

    let (done_count, set_done) = create_signal(0);
    let (memory_count, set_memory) = create_signal(0);
    let (count, set_count) = create_signal(0);

    let (error, set_error) = create_signal(String::new());

    let input_ref = create_node_ref::<Input>();

    let on_change = move |ev: leptos::ev::Event| {
        set_done(0);
        set_memory(0);
        set_error("".to_string());
        spawn_local(async move {
            set_media(Vec::new());
            let elem = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
            let files = elem.files();

            let length = files.clone().unwrap().length();
            set_count(length);

            let encoded = convert_files_to_b64(files.unwrap(), set_memory, memory_count).await;

            set_media(encoded);
        });
    };

    let on_click = move |_| {
        spawn_local(async move {
            set_done(0);
            set_count(memory_count.get_untracked() as u32);
            match upload(media.get_untracked(), set_done, done_count).await {
                Ok(_) => {
                    logging::log!("OK");
                    let input_elem = input_ref.get().unwrap();
                    input_elem.set_files(None);
                    input_elem.set_value("");

                    // Reset the media signal
                    set_media(Vec::new());
                }
                Err(e) => {
                    logging::log!("{}", e);

                    // Reset signals
                    set_memory(0);
                    set_count(0);

                    let error_message =
                        e.to_string().split(":").collect::<Vec<&str>>()[1].to_string();
                    set_error(error_message);
                }
            };
        });
    };

    let users = create_rw_signal(vec![]);
    spawn_local(async move {
        match get_user_map().await {
            Ok(u) => users.set(u),
            Err(e) => logging::log!("{}", e),
        };
    });

    view! {
        <input id="file_input" _ref=input_ref type="file" multiple="multiple" accept="image/png, image/gif, image/jpeg, image/tiff"
            on:change=on_change
        />
        <button on:click=on_click>"Upload"</button>
        <p>{ move ||
            match count() {
                0 => "".to_string(),
                _ => match done_count() {
                    0 => format!("{} / {} files ready to be uploaded", memory_count(), count()),
                    _ if done_count() < count() as usize => format!("{} / {} files uploaded", done_count(), count()),
                    _ => "The files have been successfully uploaded".to_string(),
                },
            }
        }
        </p>

        <p>{ move || error() }</p>
        <div class="upload-wrapper">
                {
                    move || if !media().is_empty() {

                        media().iter().map(|(filename, encoded_string, names)| {
                            let f = filename.clone();
                            let e = encoded_string.clone();
                            let name_list = names.clone();
                            view! {
                                    <div class="upload">
                                        <div class="horizontal">
                                            <img class="smallimg" src={format!("data:image/png;base64,{}", e)}/>
                                            <div>
                                            <For
                                                each=move || name_list.get()
                                                key=|idx| idx.id
                                                children=move |idx| {

                                                    let (get_name, set_name) = create_signal(Option::<UserInfo>::None);
                                                    let _ = create_resource(
                                                        get_name,
                                                        // every time `count` changes, this will run
                                                        move |value| async move {
                                                            let v = match value {
                                                                Some(v) => v,
                                                                None => return,
                                                            };

                                                            name_list.update(|vs| vs[idx.id].name = v.username)
                                                        },
                                                    );

                                                    view!{
                                                        <div class="horizontal person-wrapper">
                                                            <img class="profilepicture" src={format!("data:image/png;base64,{}", img_from_bounds(e.clone(), idx.bounds))} />
                                                            <OptionalSelect class="person"
                                                                options=users
                                                                search_text_provider=move |o: UserInfo| o.username
                                                                render_option=move |o: UserInfo| o.username
                                                                selected=get_name
                                                                add=move |v: String| users.update(|ns| ns.push(UserInfo{id:-1, username:v}))
                                                                set_selected=set_name
                                                                allow_deselect=true
                                                            />
                                                        </div>
                                                    }
                                                }
                                            />
                                                <button class="controls" on:click=move |_| {
                                                    name_list.update(|v| v.push(Person{name:"".to_string(), id: v.len(), bounds: None}));
                                                }>+</button>
                                                <button class="controls" on:click=move |_| {
                                                    name_list.update(|v| { v.pop(); });
                                                }>-</button>
                                            </div>
                                        </div>
                                        <button on:click=move |_| {
                                            let mut m = media.get_untracked();
                                            m.retain(|(filename, _, _)| filename != &f);
                                            set_media(m);

                                            set_memory(memory_count() - 1);

                                        }>"Remove"</button>
                                    </div>
                            }
                        }).collect::<Vec<_>>()
                    } else {
                       Vec::new()
                    }
                }
        </div>
    }
}

const FACE_PADDING: i32 = 25;
fn img_from_bounds(imgb64: String, bounds: Option<Bbox>) -> String {
    let mut image = decode_image(imgb64.clone());

    let b = match bounds {
        Some(bounds) => bounds,
        None => return imgb64,
    };

    let padding: u32 = find_padding(b.x as i32, b.y as i32, FACE_PADDING) as u32;

    let mut buf: Vec<u8> = Vec::new();
    image
        .crop(
            b.x - padding,
            b.y - padding,
            b.w + padding * 2,
            b.h + padding * 2,
        )
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .unwrap();

    base64::encode(buf)
}

fn find_padding(x: i32, y: i32, padding: i32) -> i32 {
    if x - padding >= 0 && y - padding >= 0 {
        return padding;
    }

    return find_padding(x, y, padding - 1);
}

#[cfg(feature = "ssr")]
fn extract_ext(filename: String) -> Option<String> {
    let parts = filename.split(".").collect::<Vec<_>>();
    if parts.len() > 1 {
        Some(parts[parts.len() - 1].to_string())
    } else {
        None
    }
}

async fn convert_file_to_b64(
    file: File,
    set_memory: WriteSignal<usize>,
    memory_count: ReadSignal<usize>,
) -> (String, String, RwSignal<Vec<Person>>) {
    let gloo_file = gloo::file::File::from(file);

    let bytes = gloo::file::futures::read_as_bytes(&gloo_file)
        .await
        .expect_throw("Failed to read file");
    let encoded_string = base64::encode(&bytes);
    let encoded_string_thread = encoded_string.clone();
    let names: RwSignal<Vec<Person>> = create_rw_signal(Vec::new());

    spawn_local(async move {
        let faces = match faces(encoded_string_thread).await {
            Ok(c) => c,
            Err(_err) => Vec::new(),
        };

        logging::log!("{}", faces.len());

        set_memory(memory_count.get_untracked() + 1);

        let mut names_init: Vec<Person> = Vec::new();
        for i in 0..faces.len() {
            names_init.push(Person {
                name: "".to_string(),
                id: i,
                bounds: Some(faces[i].clone()),
            });
        }

        names.set(names_init);
    });

    (gloo_file.name(), encoded_string, names)
}

async fn convert_files_to_b64(
    files: FileList,
    set_memory: WriteSignal<usize>,
    memory_count: ReadSignal<usize>,
) -> Vec<(String, String, RwSignal<Vec<Person>>)> {
    let mut res = Vec::new();
    for i in 0..files.length() {
        let file = files.get(i).expect_throw("Failed to get file");
        res.push(convert_file_to_b64(file, set_memory, memory_count));
    }

    futures::future::join_all(res).await
}
