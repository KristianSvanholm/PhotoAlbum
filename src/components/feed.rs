#[cfg(feature = "ssr")]
use crate::auth;
use crate::components::home_page::Filters;
use leptos::html::Div;
use leptos::*;
use leptonic::components::icon::Icon;
use leptos_use::{use_infinite_scroll_with_options, UseInfiniteScrollOptions};
use serde::Deserialize;
use serde::Serialize;
#[cfg(feature = "ssr")]
use std::fs::File;
#[cfg(feature = "ssr")]
use std::io::Read;

//Image struct for images from DB
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct ImageDb {
    id: String,
    path: String,
    upload_date: String,
    created_date: String,
}

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct PrevImageDb {
    created_date: String,
}

//Takes a date string and image struct
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum Element {
    String(RwSignal<String>),
    ImageDb(RwSignal<ImageDb>),
}

//Fetch images from database
#[server(Feed, "/api")]
pub async fn fetch_files(
    db_index: usize,
    count: usize,
    tags: Option<(String, Vec<String>)>,
    people: Option<(String, Vec<i64>)>,
) -> Result<Vec<Element>, ServerFnError> {
    use crate::image_filter::image_filter;
    auth::logged_in().await?;

    //DB connection
    use crate::app::ssr::*;
    let pool = pool()?;

    // Return nothing if index above limit
    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(&pool)
        .await?;
    if db_index as i64 > total_count {
        return Ok(vec![]);
    }

    let base_query = "SELECT DISTINCT f.id, f.path, f.uploadDate AS upload_date, f.createdDate AS created_date FROM files f";
    let (conditions, joins, binds) =
        image_filter::prepare_filtered_query_with_pagination(tags, people, count, db_index).await;

    let query = image_filter::build_filtered_query(
        base_query.to_string(),
        conditions,
        joins,
        Some("f.createdDate DESC".to_string()),
        Some(count),
        Some(db_index),
    );

    let mut query = sqlx::query_as(&query);
    for bind in binds {
        query = query.bind(bind);
    }

    let mut files: Vec<ImageDb> = query.fetch_all(&pool).await?;

    for img in &mut files {
        // Read the image file
        let mut file = File::open(&img.path).expect("Failed to open image file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Failed to read image file");

        // Encode the image buffer as base64
        let base64_image = base64::encode(&buffer);

        // Generate src attribute value with the base64 image
        img.path = base64_image;
    }

    let mut grouped_images: Vec<Element> = Vec::new();

    //Check previous date to prevent duplicate dates
    let mut prevfile: Option<PrevImageDb> = None;
    //Prevent checking previous date on the first request
    if db_index > 0 {
        prevfile = Some(sqlx::query_as::<_, PrevImageDb>(
            "SELECT createdDate AS created_date FROM files ORDER BY createdDate DESC LIMIT ? OFFSET ?;",
        )
        .bind(1.to_string())
        .bind((db_index-1).to_string())
        .fetch_one(&pool)
        .await?);
    }

    let mut current_month = String::new();
    let mut current_year = String::new();

    //When there is a previous date
    if prevfile.is_some() {
        //Access previous date requested
        current_month = prevfile.clone().unwrap().created_date[5..7].to_string();
        current_year = prevfile.clone().unwrap().created_date[0..4].to_string();
    }

    let mut c: i64 = 0;
    //Iterates over sorted images and adds years and months
    for image in files {
        let year = image.created_date[0..4].to_string();
        let month = image.created_date[5..7].to_string();
        if month != current_month || year != current_year {
            //Add year on change
            if year != current_year {
                grouped_images.push(Element::String(create_rw_signal(year.to_string())));
                current_year = year.to_string();
            }
            //Add month on change
            grouped_images.push(Element::String(create_rw_signal(month.to_string())));
            current_month = month.to_string();
        }
        c = c + 1;
        grouped_images.push(Element::ImageDb(create_rw_signal(image)));
    }

    Ok(grouped_images)
}

//Images per infinite feed requst
const FETCH_IMAGE_COUNT: usize = 10;

async fn request_wrapper(
    db_index: usize,
    count: usize,
    ready_lock: WriteSignal<bool>,
    tags: Option<(String, Vec<String>)>,
    people: Option<(String, Vec<i64>)>,
) -> Vec<Element> {
    ready_lock(false);
    let result = fetch_files(db_index, count, tags, people).await.unwrap();

    if !result.is_empty() {
        ready_lock(true);
    }

    result
}

//Creates an infinite feed of images
#[component]
pub fn InfiniteFeed(filter: ReadSignal<Filters>) -> impl IntoView {
    use crate::components::loading::Loading_Triangle;

    let (ready, set_ready) = create_signal(true);
    let (loading, set_loading) = create_signal(true);

    let (db_index, set_db_index) = create_signal(0);

    let initImgs: Vec<Element> = vec![];
    let (images, set_images) = create_signal(initImgs);

    let _filter_updater = create_resource(
        move || (filter.get()),
        move |_| async move {
            set_images.set(vec![]);
            set_db_index.set(0);

            let l_tags = filter.get_untracked().tags.clone();
            let l_people = filter.get_untracked().people.clone();

            let images = request_wrapper(0, FETCH_IMAGE_COUNT, set_ready, l_tags, l_people).await;
            set_images.update(|imgs| imgs.extend(images));
            set_loading(false);
        },
    );

    // Signal with resource called every time the bottom is reached in infinite feed
    let _image_updater = create_resource(
        move || (db_index.get()),
        move |_| async move {
            if db_index.get_untracked() == 0 {
                return;
            }
            let l_tags = filter.get_untracked().tags.clone();
            let l_people = filter.get_untracked().people.clone();

            let images = request_wrapper(
                db_index.get_untracked() as usize,
                FETCH_IMAGE_COUNT as usize,
                set_ready,
                l_tags,
                l_people,
            )
            .await;
            set_images.update(|imgs| imgs.extend(images));
            set_loading(false);
        },
    );

    let el: NodeRef<Div> = create_node_ref::<Div>();

    //Change feed display variables (smooth/date)
    let (name, set_name) = create_signal(icondata::BiGridSolid);
    let (feedDisplayClass, set_feedDisplayClass) = create_signal("break date_title".to_string());
    let (imageDisplayClass, set_imageDisplayClass) = create_signal("image".to_string());
    let (num, set_num) = create_signal(0);

    // //Creates and loads infinite feed
    let _ = use_infinite_scroll_with_options(
        el,
        move |_| async move {
            if !ready.get_untracked() {
                return; // TODO:: Look into disabling the entire thing instead of just returning forever
            }
            set_loading(true);

            //Index counter for DB
            let newIndex = db_index.get_untracked() + FETCH_IMAGE_COUNT;
            logging::log!(
                "Requesting {} more images. Index: {}",
                FETCH_IMAGE_COUNT,
                db_index.get_untracked() + FETCH_IMAGE_COUNT
            );
            set_db_index.set(newIndex);
        },
        UseInfiniteScrollOptions::default().distance(300.0),
    );

    view! {
        <div class="feedContainer">
            <div class="flowdiv" node_ref=el>
        //Change display of feed
                <div class="feedSettings break">
                    <button id="displayFeed" on:click=move |_| {
                        if num.get() == 0 {
                            set_name(icondata::AiBarsOutlined);
                set_feedDisplayClass("invis".to_string());
                set_imageDisplayClass("image imageSmooth".to_string());
                            set_num(1);
                        } else {
                            set_name(icondata::BiGridSolid);
                set_feedDisplayClass("break date_title".to_string());
                set_imageDisplayClass("image".to_string());
                            set_num(0);
                        }
            }><Icon icon=name/></button>
                </div>
                <For each=move || images.get() key=|i| i.clone() let:item>
                { match item{
                    //Image
                        Element::ImageDb(ref img) => {
                        view!{
                            <div class={move || imageDisplayClass.get()}>
                                    <img src={format!("data:image/jpeg;base64,{}", img.get().path)} alt="Base64 Image" class="image imageSmooth" />
                                </div>
                    }},
                    //Date
                        Element::String(ref date) => {
                    let date_clone = date.clone(); //Allow str to reach all the way in
                    view!{
                    <div class={move || feedDisplayClass.get()}>{
                                    match date_clone.get().parse().unwrap() {
                                        1 => "January".to_string(),
                                        2 => "February".to_string(),
                                        3 => "March".to_string(),
                                        4 => "April".to_string(),
                                        5 => "May".to_string(),
                                        6 => "June".to_string(),
                                        7 => "July".to_string(),
                                        8 => "August".to_string(),
                                        9 => "September".to_string(),
                                        10 => "October".to_string(),
                                        11 => "November".to_string(),
                                        12 => "December".to_string(),
                                        _ => date_clone.get()
                                    }
                                }</div>
                }}
                    }}
                </For>
                <div class="break center-h">
                <Loading_Triangle show=loading/>
                </div>
            </div>
        </div>
    }
}
