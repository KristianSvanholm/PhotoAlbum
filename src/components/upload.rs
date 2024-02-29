use leptos::*;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::*;

#[server(Upload, "/api")]
pub async fn upload_media_server(media: Vec<Vec<u8>>) -> Result<(), ServerFnError> {
    //logging::log!("files: {:?}", String::from_utf8_lossy(media));
    for bytes in media {
        logging::log!("Got data {:?}", String::from_utf8_lossy(&bytes));
    }
    Ok(())
}

#[component]
pub fn UploadMedia() -> impl IntoView {
  
    use wasm_bindgen::JsCast;

    let b: Vec<Vec<u8>> = vec![];
    let (bytes, set_bytes) = create_signal(b);
 
    let on_change = move |ev: leptos::ev::Event| {
            spawn_local(async move {
                let elem = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                let files = elem.files();
                set_bytes(file_convert(files).await);
            });        
    };

    view! {
        <input type="file" multiple="multiple"
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

async fn file_convert(files: Option<web_sys::FileList>) -> Vec<Vec<u8>> {

    let files = gloo::file::FileList::from(files.expect_throw("Empty files"));
    let mut data = vec![vec!()];

    for file in files.iter() {

        let bytes = gloo::file::futures::read_as_bytes(file)
            .await
            .expect_throw("read file");

        data.push(bytes);

    }

    data
}

//https://prestation-habitat.com/?_=%2Fleptos-rs%2Fleptos%2Fdiscussions%2F1474%23mTaCCOGt43phflpf0%2BzVKOuc
