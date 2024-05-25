#[cfg(feature = "ssr")]
use crate::auth;
use crate::components::home_page::{get_tags, Tag};
use crate::components::users::{get_user_list_sans_admin, UserInfo};
use futures::future;
use image::DynamicImage;
use leptonic::components::select::{Multiselect, OptionalSelect};
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
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Person {
    pub bounds: Option<Bbox>,
    pub name: String,
    pub id: i64,
}

#[server(Faces, "/api")]
pub async fn faces(image_b64: String) -> Result<Vec<Bbox>, ServerFnError> {
    auth::logged_in().await?;

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

pub fn decode_image(encoded_string: String) -> image::DynamicImage {
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
    people: Vec<Person>,
    tags: Vec<Tag>,
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

    // Find / create users and attach them to image.
    for person in people {
        if person.name == "".to_string() {
            continue; // Skip this person.
        }

        let user = match auth::ssr::SqlUser::get_from_username(person.name.clone(), &pool).await {
            Some(u) => u.id,
            None => {
                let res = sqlx::query("INSERT INTO users (username) VALUES (?)")
                    .bind(person.name)
                    .execute(&pool)
                    .await?;

                res.last_insert_rowid()
            }
        };

        let binds: Vec<Option<u32>> = match person.bounds {
            Some(b) => vec![b.x, b.y, b.w, b.h]
                .into_iter()
                .map(|v| Some(v))
                .collect(),
            None => vec![None::<u32>; 4],
        };

        // Insert name tags
        let mut q = sqlx::query(
            "INSERT INTO userFile (userID, fileID, x, y, width, height) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(user)
        .bind(&uuid);
        for b in binds {
            q = q.bind(b);
        }
        q.execute(&pool).await?;
    }

    // Find / create tags and attach them to image
    for mut tag in tags {
        if tag.tag_string == "".to_string() {
            continue; // Skip this tag
        }

        // Spaces not allowed in tags
        tag.tag_string = str::replace(&tag.tag_string, " ", "-");

        // Find or create new tag in db. Result is irrelevant, if it failed, the tag already
        // existed. If it succeeded, continue with new tag anyways.
        let _ = sqlx::query("INSERT INTO tags (tagString) VALUES (?)")
            .bind(&tag.tag_string)
            .bind(&uuid)
            .execute(&pool)
            .await;

        sqlx::query("INSERT INTO tagFile (tagString, fileID) VALUES (?, ?)")
            .bind(tag.tag_string)
            .bind(&uuid)
            .execute(&pool)
            .await?;
    }

    Ok(())
}

async fn upload(
    payload: Vec<(String, String, RwSignal<Vec<Person>>, RwSignal<Vec<Tag>>)>,
    set_done: WriteSignal<usize>,
    done_count: ReadSignal<usize>,
) -> Result<(), ServerFnError> {
    if payload.is_empty() {
        return Err(ServerFnError::new("No files to upload".to_string()));
    }

    let mut calls = Vec::new();

    for (filename, encoded_string, people, tags) in payload {
        calls.push(upload_wrapper(
            filename,
            encoded_string,
            people.get_untracked(),
            tags.get_untracked(),
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
    names: Vec<Person>,
    tags: Vec<Tag>,
    set_done: WriteSignal<usize>,
    done_count: ReadSignal<usize>,
) -> Result<(), ServerFnError> {
    return match upload_media_server(filename, encoded_string, names, tags).await {
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
    let tag_options = create_rw_signal(vec![]);
    spawn_local(async move {
        match get_user_list_sans_admin().await {
            Ok(u) => users.set(u),
            Err(e) => logging::log!("{}", e),
        };

        match get_tags().await {
            Ok(t) => tag_options.set(t),
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
        <div class="upload-content">
                {
                    move || if !media().is_empty() {

                        media().iter().map(|(filename, encoded_string, names, tags)| {
                            let f = filename.clone();
                            let e = encoded_string.clone();
                            let img = decode_image(e.clone());
                            let name_list = names.clone();
                            let tag_list = tags.clone();

                            let (get_tags, set_tags) = create_signal(vec![]);
                            let _ = create_resource(
                                get_tags,
                                // Every time `get_tags` changes, this will run
                                move |value| async move {
                                    tag_list.set(value);
                                },
                            );

                            view! {
                                    <div class="upload">
                                        <div class="horizontal">
                                            <img class="smallimg" src={format!("data:image/png;base64,{}", e)}/>
                                            <div class="people-scroll">
                                            <For
                                                each=move || name_list.get()
                                                key=|idx| idx.id
                                                children=move |idx| {

                                                    let (get_name, set_name) = create_signal(Option::<UserInfo>::None);
                                                    let _ = create_resource(
                                                        get_name,
                                                        // every time `get_name` changes, this will run
                                                        move |value| async move {
                                                            let v = match value {
                                                                Some(v) => v,
                                                                None => return,
                                                            };

                                                            name_list.update(|vs| vs[idx.id as usize].name = v.username)
                                                        },
                                                    );


                                                    view!{
                                                        <div class="horizontal person-wrapper">
                                                            <img class="profilepicture" src={format!("data:image/webp;base64,{}", img_from_bounds(&img, idx.bounds))} />
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
                                            </div>
                                        </div>
                                        <div class="horizontal">
                                            <button on:click=move |_| {
                                                let mut m = media.get_untracked();
                                                m.retain(|(filename, _, _, _)| filename != &f);
                                                set_media(m);

                                                set_memory(memory_count() - 1);

                                            }>"Remove"</button>
                                            <button class="controls" on:click=move |_| {
                                                name_list.update(|v| v.push(Person{name:"".to_string(), id: v.len() as i64, bounds: None}));
                                                }>+</button>
                                            <button class="controls" on:click=move |_| {
                                                name_list.update(|v| { v.pop(); });
                                            }>-</button>

                                        </div>
                                            <Multiselect class="mselect"
                                                options = tag_options
                                                search_text_provider=move |o: Tag| o.tag_string
                                                render_option=move |o: Tag| o.tag_string
                                                selected=get_tags
                                                add=move |v: String| tag_options.update(|ts| ts.push(Tag{tag_string: v}))
                                                set_selected=set_tags
                                            ></Multiselect>
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
pub fn img_from_bounds(img: &DynamicImage, bounds: Option<Bbox>) -> String {
    let mut image = img.clone();
    let mut buf: Vec<u8> = Vec::new();

    let b = match bounds {
        Some(bounds) => bounds,
        None => {
            img.write_to(
                &mut std::io::Cursor::new(&mut buf),
                image::ImageFormat::WebP,
            )
            .unwrap();

            return base64::encode(buf);
        }
    };

    let padding: u32 = find_padding(b.x as i32, b.y as i32, FACE_PADDING) as u32;

    logging::log!("{} {}.{}.{}.{}", padding, b.x, b.y, b.w, b.h);
    logging::log!("{} {}", image.width(), image.height());
    image
        .crop(
            b.x - padding,
            b.y - padding,
            b.w + padding * 2,
            b.h + padding * 2,
        )
        .write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::WebP,
        )
        .unwrap();

    base64::encode(buf)
}

fn find_padding(x: i32, y: i32, padding: i32) -> i32 {
    let x1 = x - padding;
    let y1 = y - padding;

    if x1 >= 0 && y1 >= 0 {
        return padding;
    }

    // Whichever is the smallest,
    // "add" that (negative values, so it will subtract)
    if x1 < y1 {
        return padding + x1;
    } else {
        return padding + y1;
    }
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
) -> (String, String, RwSignal<Vec<Person>>, RwSignal<Vec<Tag>>) {
    let gloo_file = gloo::file::File::from(file);

    let bytes = gloo::file::futures::read_as_bytes(&gloo_file)
        .await
        .expect_throw("Failed to read file");
    let encoded_string = base64::encode(&bytes);
    let encoded_string_thread = encoded_string.clone();
    let names: RwSignal<Vec<Person>> = create_rw_signal(Vec::new());
    let tags: RwSignal<Vec<Tag>> = create_rw_signal(Vec::new());

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
                id: i as i64,
                bounds: Some(faces[i].clone()),
            });
        }

        names.set(names_init);
    });

    (gloo_file.name(), encoded_string, names, tags)
}

async fn convert_files_to_b64(
    files: FileList,
    set_memory: WriteSignal<usize>,
    memory_count: ReadSignal<usize>,
) -> Vec<(String, String, RwSignal<Vec<Person>>, RwSignal<Vec<Tag>>)> {
    let mut res = Vec::new();
    for i in 0..files.length() {
        let file = files.get(i).expect_throw("Failed to get file");
        res.push(convert_file_to_b64(file, set_memory, memory_count));
    }

    futures::future::join_all(res).await
}
