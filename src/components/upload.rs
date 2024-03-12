use leptos::*;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::*;
use rand::Rng;


#[derive(Debug, Clone, leptos::server_fn::serde::Serialize, leptos::server_fn::serde::Deserialize)]
pub struct MediaPayload {
    data: Vec<(String, Vec<u8>)>
}

#[server(Upload, "/api")]
pub async fn upload_media_server(media: MediaPayload) -> Result<(), ServerFnError> {
    use std::fs;
    use std::path::Path;
    use crate::app::ssr::*;

    let pool = pool()?;

    if !Path::new("./album").exists() {
        let _ = fs::create_dir_all("./album")?;
    }

    for (filename, bytes) in media.data {
        use uuid::Uuid;
        let file_ext = match extract_ext(filename.clone()) {
            Some(ext) => ext,
            None => continue,
        };

        let uuid = Uuid::new_v4().to_string();
        let base64_data = base64::encode(&bytes); //Convert image to base64
    
        sqlx::query("INSERT INTO files (id, path, uploadDate, createdDate) VALUES (?, ?, ?, ?)")
            .bind(uuid)
            .bind(base64_data) // Bind base64 data to the path field
            .bind("fake_timestamp".to_string())
            //Randomize date, only for testing
            .bind(format!(
                "{}-{:02}-{:02}",
                rand::thread_rng().gen_range(2010..2023),
                rand::thread_rng().gen_range(1..13),
                rand::thread_rng().gen_range(1..29),
            ).to_string())
            .execute(&pool)
            .await?;

    }
    Ok(())
}

#[component]
pub fn UploadMedia() -> impl IntoView {
  
    use wasm_bindgen::JsCast;

    let b = MediaPayload {data: vec!()};

    let (bytes, set_bytes) = create_signal(b);
 
    let on_change = move |ev: leptos::ev::Event| {
            spawn_local(async move {
                let elem = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                let files = elem.files();
                set_bytes(file_convert(files).await);
            });        
    };

    view! {
        <input type="file" multiple="multiple" accept="image/png, image/gif, image/jpeg"
            on:change=on_change
        />
        <button on:click=move |_| {
            spawn_local(async move {
                match upload_media_server(bytes.get_untracked()).await {
                    Ok(_) => logging::log!("OK"),
                    Err(e) => logging::log!("{}", e),
                };
            });
        }>"Upload"</button>
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

async fn file_convert(files: Option<web_sys::FileList>) -> MediaPayload {

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

    media
}
