use leptos::*;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::*;
use futures::{executor, future, FutureExt}; // 0.3.5

#[derive(Debug, Clone, leptos::server_fn::serde::Serialize, leptos::server_fn::serde::Deserialize)]
pub struct MediaPayload {
    data: Vec<(String, Vec<u8>)>
}

#[server(Upload, "/api")]
pub async fn upload_media_server(filename: String, bytes: Vec<u8>) -> Result<(), ServerFnError> {
    use std::fs;
    use std::path::Path;
    use crate::app::ssr::*;
    use rand::Rng;

    let pool = pool()?;

    if !Path::new("./album").exists() {
        let _ = fs::create_dir_all("./album")?;
    }

    use uuid::Uuid;
    let file_ext = extract_ext(filename).expect_throw("Missing file extension");
    let uuid = Uuid::new_v4().to_string();

    let path = format!("./album/{}.{}", uuid, file_ext);
    let _ = fs::write(&path, bytes)?;
    
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

async fn upload(media: MediaPayload, set_done: WriteSignal<usize>, done_count: ReadSignal<usize> ) -> Result<(), ServerFnError>{
    let mut calls = Vec::new(); 
        for (filename, bytes) in media.data {
        calls.push(upload_wrapper(filename, bytes, set_done, done_count));
    }

    let _results = future::join_all(calls).await;

    Ok(())
}

async fn upload_wrapper(
        filename: String, 
        bytes: Vec<u8>, 
        set_done: WriteSignal<usize>, 
        done_count: ReadSignal<usize>
    ) -> Result<(), ServerFnError>{

    return match upload_media_server(filename, bytes).await {
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

    let b = MediaPayload {data: vec!()};

    let (bytes, set_bytes) = create_signal(b);


    let (done_count, set_done) = create_signal(0);
    let (count, set_count) = create_signal(0);
 
    let on_change = move |ev: leptos::ev::Event| {
            spawn_local(async move {
                let elem = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                let files = elem.files();
                let (bs, c) = file_convert(files).await;
                set_count(c);
                set_bytes(bs);
            });        
    };

    view! {
        <input type="file" multiple="multiple" accept="image/png, image/gif, image/jpeg"
            on:change=on_change
        />
        <button on:click=move |_| {
            spawn_local(async move {
                match upload(bytes.get_untracked(), set_done, done_count).await {
                    Ok(_) => logging::log!("OK"),
                    Err(e) => logging::log!("{}", e),
                };
            });
        }>"Upload"</button>
        <p>{ move || done_count()} / {move || count()}</p>
    }
}

#[cfg(feature = "ssr")]
fn extract_ext(filename: String) -> Option<String> {
    let parts = filename.split(".").collect::<Vec<_>>();
    let n = parts.len();
    if n < 2 {
        return None;
    }
    Some(parts[n-1].to_string())
}

async fn file_convert(files: Option<web_sys::FileList>) -> (MediaPayload, usize) {

    let files = gloo::file::FileList::from(files.expect_throw("Empty files"));
    let mut media = MediaPayload {
        data: vec!(),
    };

    for file in files.iter() {

        let bytes = gloo::file::futures::read_as_bytes(file)
            .await
            .expect_throw("read file");

        media.data.push((file.name(),bytes));

    }

    (media, files.len())
}
