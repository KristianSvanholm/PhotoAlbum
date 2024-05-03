use leptos::*;
use crate::components::feed::InfiniteFeed;
use crate::components::upload::UploadMedia;
use crate::components::image_view::ImageView;
use crate::components::dialog::Dialog;

#[server(ImageList, "/api")]
pub async fn fetch_image_list() -> Result<Vec<String>, ServerFnError> {
    use crate::auth;
    auth::logged_in().await?;

    //DB connection
    use crate::app::ssr::*;
    let pool = pool()?;

    //Fetch images in descending order
    //TODO: add filtering here 
    let img_ids = sqlx::query_scalar(
        "SELECT id FROM files ORDER BY uploadDate ASC;",
    )
    .fetch_all(&pool)
    .await?;

    Ok(img_ids)
}

#[server(NextImageId, "/api")]
pub async fn next_prev_image_id(prev_id: String, offset: i16) -> Result<String, ServerFnError> {
    use crate::auth;
    auth::logged_in().await?;

    //DB connection
    use crate::app::ssr::*;
    let pool = pool()?;

    let new_id = sqlx::query_scalar(
        "SELECT id FROM(
            SELECT id,
            ROW_NUMBER () OVER ( 
                ORDER BY uploadDate DESC
            ) RowNum
            FROM files)
        WHERE RowNum = (SELECT RowNum+? FROM 
                (SELECT id,
                ROW_NUMBER () OVER ( 
                    ORDER BY uploadDate DESC
                ) RowNum
                FROM files)
        WHERE id = ?);",
    )
    .bind(offset)
    .bind(prev_id)
    .fetch_one(&pool)
    .await?;

    Ok(new_id)
}

#[component]
pub fn HomePage() -> impl IntoView
{
    let (showing_upload, set_showing_upload) = create_signal(false);
    let (image_id, set_image_id) = create_signal(None); 
    //let images = create_resource(|| (), |_| async move {fetch_image_list()});

    let next_prev_image_click = move |offset:i16| {
        spawn_local(async move {
            match next_prev_image_id(image_id.get_untracked().unwrap_or_default(), offset).await {
                Ok(next_id) => 
                {
                    set_image_id(Some(next_id));
                },
                Err(e) => {
                    logging::log!("{}", e);
                    let error_message = e.to_string().split(":").collect::<Vec<&str>>()[1].to_string();
                },
            };
        })
    };

    view! {
        <button
            class = "floating displayFeed"
            on:click=move |_| {
                logging::log!("Open upload dialog"); 
                set_showing_upload(true);
            }><i class="fas fa-plus"></i>
        </button>
        <Dialog 
            on_close=move || set_image_id(None)
            open=move || image_id.get().is_some()
            close_on_outside=true
            close_button=false>
            <ImageView image_id=move || image_id.get().unwrap_or_default()/>
            <div class="bottom-buttons">
                <button on:click=move |_| next_prev_image_click(-1)><i class="fas fa-angle-left"></i></button>
                <button on:click=move |_| set_image_id(None)>"Close"</button>
                <button on:click=move |_| next_prev_image_click(1)>
                    <i class="fas fa-angle-right"></i>
                </button>
            </div>
        </Dialog>
        <Dialog 
            on_close=move || set_showing_upload(false) 
            open=showing_upload>
            <h1>"Upload"</h1>
            <UploadMedia/>
        </Dialog>
        <InfiniteFeed on_image_click=move |image_id:String| set_image_id(Some(image_id))/>
    }
}