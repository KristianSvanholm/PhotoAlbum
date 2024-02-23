use leptos::{html::Input, *};
use web_sys::*;

#[component]
pub fn UploadMedia() -> impl IntoView {
  
    use wasm_bindgen::JsCast;

    let media_input: NodeRef<html::Input> = create_node_ref();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        let value = media_input().expect("<Input> should be mounted").value();
        logging::log!("{}", value);
    };
 
    let on_change = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);

        let elem = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
        // .files() dont exist??
        let files = elem.files();
        logging::log!("value: {:?} files: {:?}", val, files);
    };

    view! {
        <form on:submit=on_submit>
            <input type="file"
                node_ref=media_input
                on:change=on_change
            />
            <input type="submit" value="Submit"/>
        </form>
    }

}

//https://prestation-habitat.com/?_=%2Fleptos-rs%2Fleptos%2Fdiscussions%2F1474%23mTaCCOGt43phflpf0%2BzVKOuc
