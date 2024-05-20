use leptos::*;
use crate::components::feed::InfiniteFeed;
use crate::components::upload::UploadMedia;
use crate::components::image_view::ImageView;
use crate::components::dialog::Dialog;
use crate::auth;
use std::fs;
use serde::Deserialize;
use serde::Serialize;

//Image struct for images from DB
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct ImageDb {
    path: String,
}

#[server(NextImageId, "/api")]
pub async fn next_prev_image_id(prev_id: String, offset: i16) -> Result<Option<String>, ServerFnError> {
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
    .fetch_optional(&pool)
    .await?;

    Ok(new_id)
}

//Delete image from database
#[server(DeleteImage, "/api")]
pub async fn delete_image(image_id: String) -> Result<(), ServerFnError> {
    let user = auth::logged_in().await?;
    let admin = auth::authorized("admin").await;
    
    //DB connection
    use crate::db::ssr::pool;
    let pool = pool()?;


    //Check if user is uploader
    let uploader:bool = sqlx::query_scalar("SELECT uploadedBy=? FROM files WHERE id = ?")
        .bind(user.id)
        .bind(image_id.clone())
        .fetch_one(&pool)
        .await?;

    //Check for uploader or admin access
    if !uploader {
        match admin {
            Ok(_) => {}
            Err(_) => {
                return Err(ServerFnError::ServerError(
                    "You are not authorized to delete this image".to_string(),
                ))
            }
        }
    }

    //Fetch image
    let img = sqlx::query_as::<_, ImageDb>(
        "SELECT path FROM files WHERE id = ?;",
    )
    .bind(image_id.to_string())
    .fetch_one(&pool)
    .await?;

    //Delete from database
    sqlx::query(
        "DELETE FROM files WHERE id = ?;",
    )
    .bind(image_id.clone())
    .execute(&pool)
    .await?;

    //Delete from file system
    fs::remove_file(img.path)?;

    Ok(())
}

#[component]
pub fn HomePage() -> impl IntoView
{
    let (showing_upload, set_showing_upload) = create_signal(false);
    let (image_id, set_image_id) = create_signal(None);
    let next_image_id = create_local_resource(
        image_id, 
        |prev_id| async move {
            if prev_id.is_some(){ match next_prev_image_id(prev_id.unwrap(), 1).await{
                Ok(Some(id))=>Some(id),
                Ok(None) => None,
                Err(_err) => None,
            }}
            else {None}
        }
    ); 
    let prev_image_id = create_local_resource(
        image_id, 
        |prev_id| async move {
            if prev_id.is_some(){ match next_prev_image_id(prev_id.unwrap(), -1).await{
                Ok(Some(id))=>Some(id),
                Ok(None) => None,
                Err(_err) => None,
            }}
            else {None}
        }
    );

    let del_image = create_action(|image_id: &String| {delete_image(image_id.to_string())});
    let (del_image_id, set_del_image_id) = create_signal(Some("aaaaa".to_string()));

    let delete_action = create_action(
        move |_| async move {
            //Send signal to feed for image deletion
            set_del_image_id(image_id.get());
            //Initiate deletion
            del_image.dispatch(image_id.get().unwrap_or_default());
            //Set to next or previous image after deletion, or close 
            if !next_image_id.get().is_none() && !next_image_id.get().unwrap().is_none() {
                set_image_id(next_image_id.get().unwrap());
            } else if !prev_image_id.get().is_none() && !prev_image_id.get().unwrap().is_none() {
                set_image_id(prev_image_id.get().unwrap());
            } else {
                set_image_id(None);
            }
        }
    );

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
            
            <ImageView image_id=move || image_id.get().unwrap_or_default() push_delete=delete_action/>

            <div class="bottom-buttons">
                <button on:click=move |_| set_image_id(prev_image_id.get().unwrap())
                    disabled=move||{prev_image_id.get().is_none() || prev_image_id.get().unwrap().is_none()}>
                    <i class="fas fa-angle-left"></i></button>
                <button on:click=move |_| set_image_id(None)>"Close"</button>
                <button on:click=move |_| set_image_id(next_image_id.get().unwrap())
                    disabled=move||{next_image_id.get().is_none() || next_image_id.get().unwrap().is_none()}>
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

        <InfiniteFeed 
        on_image_click=move |image_id:String| set_image_id(Some(image_id)) 
        send_id=del_image_id
        />
        
    }
}