#[cfg(feature = "ssr")]
use crate::auth;
use futures::future; // 0.3.5
use image::{DynamicImage, GrayImage};
use leptos::{html::Input, *};
use rustface::{Detector, ImageData};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::*;

#[derive(
    Debug, Clone, leptos::server_fn::serde::Serialize, leptos::server_fn::serde::Deserialize,
)]
pub struct MediaPayload {
    data: Vec<(String, Vec<u8>)>,
}

#[server(Faces, "/api", "Cbor")]
pub async fn faces(image_b64: String) -> Result<usize, ServerFnError> {
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
    let facecount = detect_faces(&mut *detector, &image);
    Ok(facecount)
}

#[cfg(feature = "ssr")]
fn decode_image(encoded_string: String) -> DynamicImage {
    let bytes = base64::decode(encoded_string).expect("Failed to decode image");
    image::load_from_memory(&bytes).expect("Failed to load image")
}

#[cfg(feature = "ssr")]
fn detect_faces(detector: &mut dyn Detector, gray: &GrayImage) -> usize {
    let (width, height) = gray.dimensions();
    let image = ImageData::new(gray, width, height);
    let faces = detector.detect(&image);
    faces.len()
}

#[server(Upload, "/api", "Cbor")]
pub async fn upload_media_server(
    filename: String,
    encoded_string: String,
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
    .bind(uuid)
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

    Ok(())
}

async fn upload(
    payload: Vec<(String, String, usize)>,
    set_done: WriteSignal<usize>,
    done_count: ReadSignal<usize>,
) -> Result<(), ServerFnError> {
    if payload.is_empty() {
        return Err(ServerFnError::new("No files to upload".to_string()));
    }

    let mut calls = Vec::new();

    for (filename, encoded_string, _) in payload {
        calls.push(upload_wrapper(
            filename,
            encoded_string,
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
    set_done: WriteSignal<usize>,
    done_count: ReadSignal<usize>,
) -> Result<(), ServerFnError> {
    return match upload_media_server(filename, encoded_string).await {
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

                        media().iter().map(|(filename, encoded_string, name_count)| {
                            let f = filename.clone();
                            let e = encoded_string.clone();
                            let nc = name_count.clone();
                            let (count, set_count) = create_signal(nc);

                            view! {
                                    <div class="upload">
                                        <div class="horizontal">
                                            <img src={format!("data:image/png;base64,{}", e)}/>
                                            <div>
                                            <For
                                                each=move || 1..=count.get()
                                                key=|idx| *idx
                                                children=move |_| {
                                                    view!{
                                                        <input></input><br/>
                                                    }
                                                }
                                            />
                                                <button class="controls" on:click=move |_| {
                                                    set_count.update(|n| *n += 1);
                                                }>+</button>
                                                <button class="controls" on:click=move |_| {
                                                    set_count.update(|n| *n -= 1);
                                                }>-</button>

                                            /*{
                                                for _n in 0..count.get() {
                                                    return view! {
                                                        <div>
                                                            <input></input><br/>
                                                         </div>
                                                        }
                                                }


                                            }*/
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
) -> (String, String, usize) {
    let gloo_file = gloo::file::File::from(file);

    let bytes = gloo::file::futures::read_as_bytes(&gloo_file)
        .await
        .expect_throw("Failed to read file");
    let encoded_string = base64::encode(&bytes);

    let count = match faces(encoded_string.clone()).await {
        Ok(c) => c,
        Err(_err) => 0, // Handle error?
    }; // face count

    logging::log!("{}", count);

    set_memory(memory_count.get_untracked() + 1);
    (gloo_file.name(), encoded_string, count)
}

async fn convert_files_to_b64(
    files: FileList,
    set_memory: WriteSignal<usize>,
    memory_count: ReadSignal<usize>,
) -> Vec<(String, String, usize)> {
    let mut res = Vec::new();
    for i in 0..files.length() {
        let file = files.get(i).expect_throw("Failed to get file");
        res.push(convert_file_to_b64(file, set_memory, memory_count));
    }

    futures::future::join_all(res).await
}
