use leptos::*;
use crate::components::feed::InfiniteFeed;
use crate::components::upload::UploadMedia;
use crate::components::image_view::ImageView;
use crate::components::dialog::Dialog;

#[component]
pub fn HomePage() -> impl IntoView
{
    let (showing_upload, set_showing_upload) = create_signal(false);
    let (showing_image, set_showing_image) = create_signal(true);  

    view! {
        <button
            class = "floating displayFeed"
            on:click=move |_| {
                logging::log!("Open upload dialog"); 
                set_showing_upload(true);
            }><i class="fas fa-plus"></i>
        </button>
        <Dialog 
            on_close=move || set_showing_image(false)
            open=showing_image
            close_on_outside=true
            close_button=false>
            <ImageView image_id="4655b97b-af2d-40a8-8ecb-37857d425c64".to_string()/>
            <div class="bottom-buttons">
                <button><i class="fas fa-angle-left"></i></button>
                <button>"Close"</button>
                <button><i class="fas fa-angle-right"></i></button>
            </div>
        </Dialog>
        <Dialog 
            on_close=move || set_showing_upload(false) 
            open=showing_upload>
            <h1>"Upload"</h1>
            <UploadMedia/>
        </Dialog>
        <InfiniteFeed/>
    }
}