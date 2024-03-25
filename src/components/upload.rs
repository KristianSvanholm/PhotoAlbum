use leptos::*;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::*;
use futures::future; // 0.3.5

#[derive(Debug, Clone, leptos::server_fn::serde::Serialize, leptos::server_fn::serde::Deserialize)]
pub struct MediaPayload {
    data: Vec<(String, Vec<u8>)>
}

#[server(Upload, "/api", "Cbor")]
pub async fn upload_media_server(filename: String, encoded_string: String) -> Result<(), ServerFnError> {
    use std::fs;
    use std::path::Path;
    use crate::db::ssr::pool;
    use rand::Rng;

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
    
    sqlx::query("INSERT INTO files (id, path, uploadDate, createdDate) VALUES (?, ?, ?, ?)")
        .bind(uuid)
        .bind(path)
        .bind("fake_timestamp".to_string())
        //Randomize data for testing
        .bind(format!(
            "{}-{:02}-{:02}",
            rand::thread_rng().gen_range(2010..2023),
            rand::thread_rng().gen_range(1..13),
            rand::thread_rng().gen_range(1..29),
        ).to_string())
        .execute(&pool)
        .await?;

    
    Ok(())
}

async fn upload(payload: Vec<(String, String)>, set_done: WriteSignal<usize>, done_count: ReadSignal<usize> ) -> Result<(), ServerFnError>{
    let mut calls = Vec::new(); 
        for (filename, encoded_string) in payload {
            calls.push(upload_wrapper(filename, encoded_string, set_done, done_count));
    }

    let _results = future::join_all(calls).await;

    Ok(())
}

async fn upload_wrapper(
        filename: String, 
        encoded_string: String, 
        set_done: WriteSignal<usize>, 
        done_count: ReadSignal<usize>
    ) -> Result<(), ServerFnError>{

    return match upload_media_server(filename, encoded_string).await {
        Ok(_) => {
            set_done(done_count.get_untracked()+1);
            logging::log!("{}", done_count.get_untracked());
            Ok(())
        },
        Err(e) => Err(e)
    };
}

#[component]
pub fn UploadMedia() -> impl IntoView {
  
    use wasm_bindgen::JsCast;
    
    let (media, set_media) = create_signal(Vec::new());

    let (done_count, set_done) = create_signal(0);
    let (memory_count, set_memory) = create_signal(0);
    let (count, set_count) = create_signal(0);
 
    let on_change = move |ev: leptos::ev::Event| {
            set_done(0);
            set_memory(0);
            spawn_local(async move {
                let elem = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                let files = elem.files();
                let length = files.clone().unwrap().length();
                set_count(length);
                let encoded = convert_files_to_b64(files.unwrap(), set_memory, memory_count).await;
                set_media(encoded);
            });        
    };

    view! {
        <input type="file" multiple="multiple" accept="image/png, image/gif, image/jpeg, image/tiff"
            on:change=on_change
        />
        <button on:click=move |_| {
            spawn_local(async move {
                set_done(0);
                match upload(media.get_untracked(), set_done, done_count).await {
                    Ok(_) => logging::log!("OK"),
                    Err(e) => logging::log!("{}", e),
                };
            });
        }>"Upload"</button>
        <p>{ move || memory_count()} / {move || count()} files read.</p>
        <p>{ move || done_count()} / {move || count()} finished uploading.</p>
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
    memory_count: ReadSignal<usize>
) -> (String, String) {
    let gloo_file = gloo::file::File::from(file);
    let bytes = gloo::file::futures::read_as_bytes(&gloo_file).await.expect_throw("Failed to read file");
    let encoded_string = base64::encode(&bytes);
    set_memory(memory_count.get_untracked() + 1);
    (gloo_file.name(), encoded_string)
}

async fn convert_files_to_b64(
    files: FileList, 
    set_memory: WriteSignal<usize>, 
    memory_count: ReadSignal<usize>
) -> Vec<(String, String)> {
    let mut res = Vec::new();
    for i in 0..files.length() {
        let file = files.get(i).expect_throw("Failed to get file");
        res.push(convert_file_to_b64(file, set_memory, memory_count));
    }

    futures::future::join_all(res).await
}
