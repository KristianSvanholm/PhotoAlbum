use crate::components::dialog::Dialog;
use crate::components::feed::InfiniteFeed;
use crate::components::image_view::ImageView;
use crate::components::upload::UploadMedia;
use leptos::*;
#[cfg(feature = "ssr")]
use std::fs;

#[cfg(feature = "ssr")]
use crate::auth;
use leptonic::components::icon::Icon;
use leptonic::components::select::Multiselect;
use leptos::html::Select;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub struct Filters {
    pub tags: Option<(String, Vec<String>)>,
    pub people: Option<(String, Vec<i64>)>,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub tag_string: String,
}

#[server(GetTags, "/api")]
pub async fn get_tags() -> Result<Vec<Tag>, ServerFnError> {
    auth::logged_in().await?;
    use crate::db::ssr::pool;
    let pool = pool()?;

    let tags = sqlx::query_as::<_, Tag>("SELECT tagString as tag_string FROM tags")
        .fetch_all(&pool)
        .await?;

    Ok(tags)
}

//Image struct for images from DB
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct ImageDb {
    path: String,
}

#[server(NextImageId, "/api")]
pub async fn next_prev_image_id(
    prev_id: String,
    offset: i16,
) -> Result<Option<String>, ServerFnError> {
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
    use crate::auth;
    let user = auth::logged_in().await?;
    let admin = auth::authorized("admin").await;

    //DB connection
    use crate::db::ssr::pool;
    let pool = pool()?;

    //Check if user is uploader
    let uploader: bool = sqlx::query_scalar("SELECT uploadedBy=? FROM files WHERE id = ?")
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
    let img = sqlx::query_as::<_, ImageDb>("SELECT path FROM files WHERE id = ?;")
        .bind(image_id.to_string())
        .fetch_one(&pool)
        .await?;

    //Delete from database
    sqlx::query("DELETE FROM files WHERE id = ?;")
        .bind(image_id.clone())
        .execute(&pool)
        .await?;

    //Delete from file system
    fs::remove_file(img.path)?;

    Ok(())
}

#[component]
pub fn HomePage() -> impl IntoView {
    let (showing_upload, set_showing_upload) = create_signal(false);
    let (image_id, set_image_id) = create_signal(None);
    let next_image_id = create_local_resource(image_id, |prev_id| async move {
        if prev_id.is_some() {
            match next_prev_image_id(prev_id.unwrap(), 1).await {
                Ok(Some(id)) => Some(id),
                Ok(None) => None,
                Err(_err) => None,
            }
        } else {
            None
        }
    });
    let prev_image_id = create_local_resource(image_id, |prev_id| async move {
        if prev_id.is_some() {
            match next_prev_image_id(prev_id.unwrap(), -1).await {
                Ok(Some(id)) => Some(id),
                Ok(None) => None,
                Err(_err) => None,
            }
        } else {
            None
        }
    });

    let (del_image_from_feed, set_del_image_from_feed) = create_signal(None::<String>);

    let delete_image = move |_| {
        spawn_local(async move {
            //Initiate deletion
            match delete_image(image_id.get_untracked().unwrap_or_default().to_string()).await {
                Ok(_) => {
                    //Send signal to feed for image deletion
                    set_del_image_from_feed(image_id.get_untracked());
                    //Set to next or previous image after deletion, or close
                    if !next_image_id.get().is_none() && !next_image_id.get().unwrap().is_none() {
                        set_image_id(next_image_id.get().unwrap());
                    } else if !prev_image_id.get().is_none()
                        && !prev_image_id.get().unwrap().is_none()
                    {
                        set_image_id(prev_image_id.get().unwrap());
                    } else {
                        set_image_id(None);
                    }
                }
                Err(_) => {
                    //Handle error
                }
            }
        });
    };

    let filter_options = vec!["HAS".to_string(), "ONLY".to_string(), "NOT".to_string()];

    let (filter, set_filter) = create_signal(Filters {
        tags: None,
        people: None,
    });

    let users = create_rw_signal(vec![]);
    let tags = create_rw_signal(vec![]);
    spawn_local(async move {
        match crate::components::users::get_user_list_sans_admin().await {
            Ok(m) => users.set(m),
            Err(e) => logging::log!("{}", e),
        };

        match get_tags().await {
            Ok(t) => tags.set(t),
            Err(e) => logging::log!("{}", e),
        };

        logging::log!("{:?}", tags.get_untracked());
    });

    let select_ref = create_node_ref::<Select>();
    let select_ref_2 = create_node_ref::<Select>();

    let selected_users = create_rw_signal(vec![]);
    let selected_tags = create_rw_signal(vec![]);

    view! {
        <button
            class = "floating displayFeed"
            on:click=move |_| {
                logging::log!("Open upload dialog");
                set_showing_upload(true);
            }><Icon class="icon" icon=icondata::FaPlusSolid/>
        </button>
        <Dialog
            on_close=move || set_image_id(None)
            open=move || image_id.get().is_some()
            close_on_outside=true
            close_button=false>

            <ImageView
                image_id=move || image_id.get().unwrap_or_default()
                push_delete=delete_image/>


            //Mobile devices
            <div class="mobile_buttons bottom-buttons">
                <button on:click=move |_| set_image_id(prev_image_id.get().unwrap())
                    disabled=move||{prev_image_id.get().is_none() || prev_image_id.get().unwrap().is_none()}>
                    <Icon class="icon" icon=icondata::FaAngleLeftSolid /></button>
                <button on:click=move |_| set_image_id(None)>"Close"</button>
                <button on:click=move |_| set_image_id(next_image_id.get().unwrap())
                    disabled=move||{next_image_id.get().is_none() || next_image_id.get().unwrap().is_none()}>
                    <Icon class="icon" icon=icondata::FaAngleRightSolid />
                </button>
            </div>
            //PC devies
            <button class="bottom-buttons-left" on:click=move |_| set_image_id(prev_image_id.get().unwrap())
                disabled=move||{prev_image_id.get().is_none() || prev_image_id.get().unwrap().is_none()}>
                <Icon class="icon" icon=icondata::FaAngleLeftSolid /></button>
            <button class="bottom-buttons-close" on:click=move |_| set_image_id(None)>"X"</button>
            <button  class="bottom-buttons-right" on:click=move |_| set_image_id(next_image_id.get().unwrap())
                disabled=move||{next_image_id.get().is_none() || next_image_id.get().unwrap().is_none()}>
                <Icon class="icon" icon=icondata::FaAngleRightSolid />
            </button>
        </Dialog>
        <Dialog
            on_close=move || set_showing_upload(false)
            open=showing_upload>
            <h1>"Upload"</h1>
            <UploadMedia/>
        </Dialog>

        <div class="horizontal">
        <select
            _ref=select_ref
            >
            {filter_options.iter().map(|option| {
                view! {
                    <option value={option.clone()}>{option}</option>
                }
            }).collect::<Vec<_>>()}
        </select>
        <Multiselect class="mselect"
            options = tags
            search_text_provider=move |o: Tag| o.tag_string
            render_option=move |o: Tag| o.tag_string
            selected=selected_tags
            set_selected=move |v| selected_tags.set(v)
        ></Multiselect>
        <select
            _ref=select_ref_2
            >
            {filter_options.iter().map(|option| {
                view! {
                    <option value={option.clone()}>{option}</option>
                }
            }).collect::<Vec<_>>()}
        </select>
        <Multiselect class="mselect"
            options = users
            search_text_provider=move |o: crate::components::users::UserInfo| o.username
            render_option=move |o: crate::components::users::UserInfo| o.username
            selected=selected_users
            add=move |x| logging::log!("{}", x)
            set_selected=move |v| selected_users.set(v)
        ></Multiselect>
        <button
            on:click=move |_| {
                let filter = select_ref.get().unwrap().value();
                let filter_2 = select_ref_2.get().unwrap().value();
                let i_tags: Vec<String> = selected_tags.get_untracked().into_iter().map(|x: Tag| x.tag_string).collect();
                let i_people: Vec<i64> = selected_users.get_untracked().into_iter().map(|x: crate::components::users::UserInfo| x.id).collect();

                let mut valid_tag_filter: Option<(String, Vec<String>)> = Some((filter, i_tags.clone()));
                let mut valid_people_filter: Option<(String, Vec<i64>)> = Some((filter_2, i_people.clone()));


                if i_tags.is_empty() || i_tags.iter().all(|tag| tag == ""){
                    valid_tag_filter = None;
                }

                if i_people.is_empty() || i_people.iter().all(|person| *person == 0){
                    valid_people_filter = None;
                }

                set_filter(Filters {
                    tags: valid_tag_filter,
                    people: valid_people_filter,
                });

            }
            >"Filter"
        </button>
        </div>
        <InfiniteFeed
            on_image_click=move |image_id:String|
            set_image_id(Some(image_id))
            send_id=del_image_from_feed
            filter=filter
        />
    }
}
