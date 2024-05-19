use crate::components::dialog::Dialog;
use crate::components::feed::InfiniteFeed;
use crate::components::upload::UploadMedia;
use leptonic::components::select::Multiselect;
use leptonic::components::icon::Icon;
use leptos::html::Select;
use leptos::*;
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
    use crate::db::ssr::pool;
    let pool = pool()?;

    let tags = sqlx::query_as::<_, Tag>("SELECT tagString as tag_string FROM tags")
        .fetch_all(&pool)
        .await?;

    Ok(tags)
}

#[component]
pub fn HomePage() -> impl IntoView {
    let (showing_upload, set_showing_upload) = create_signal(false);

    let filter_options = vec!["HAS".to_string(), "ONLY".to_string(), "NOT".to_string()];

    let (filter, set_filter) = create_signal(Filters {
        tags: None,
        people: None,
    });

    let users = create_rw_signal(vec![]);
    let tags = create_rw_signal(vec![]);
    spawn_local(async move {
        match crate::components::users::get_user_map().await {
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
            }><Icon icon=icondata::FaPlusSolid/></button>
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
        <InfiniteFeed filter=filter/>
    }
}
