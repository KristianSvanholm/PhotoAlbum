use leptos::*;
use leptos::html::Div;
use leptos_use::{UseInfiniteScrollOptions, use_infinite_scroll_with_options};
use serde::Serialize;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

//Image struct for images from DB
#[cfg_attr(feature="ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct ImageDb {
    id: String,
    path: String,
    upload_date: String,
    created_date: String,
}


//Fetch images from database
#[server(Feed, "/api")]
pub async fn fetch_files(db_index: usize, count: usize) -> Result<Vec<ImageDb>, ServerFnError> {
    //DB connection
    use crate::app::ssr::*;
    let pool = pool()?; 

    //Stop requests at max (Work in progress... (does not really do what i want it to do))
    //I think we need to stop the client before it reaches here, since the infinite feed
    //spams requests in here, which cant be good
    if db_index as i64 > 23 {
        return Ok(vec![]);
    }
    // let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
    //     .fetch_one(&pool)
    //     .await?;

    //Fetch images in descending order
    let mut files = sqlx::query_as::<_, ImageDb>(
        "SELECT id, path, uploadDate AS upload_date, createdDate AS created_date FROM files ORDER BY createdDate DESC LIMIT ? OFFSET ?;",
    )
    .bind(count.to_string())
    .bind(db_index.to_string())
    .fetch_all(&pool)
    .await?;

    for img in &mut files{
        // Read the image file
        let mut file = File::open(&img.path).expect("Failed to open image file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read image file");

        // Encode the image buffer as base64
        let base64_image = base64::encode(&buffer);

        // Generate src attribute value with the base64 image
        img.path = base64_image;
    }

    Ok(files)
}

//Helper function to build the vector of images as they are requested from db
pub async fn fetch_files_and_handle_errors(db_index: usize, count: usize) -> Vec<ImageDb> {
    match fetch_files(db_index, count).await {
        //Extend the current vector if ok
        Ok(files) => files, 
        //Returns the incoming vector, e.g. if the there are no more images in db
        Err(_) => vec![],
    }
}

async fn append_imgs(y: WriteSignal<Vec<ImageDb>>, x: ReadSignal<Vec<ImageDb>>, additional: Vec<ImageDb>, stop: WriteSignal<bool>) -> Vec<ImageDb>{
    if additional.len() == 0 {
        stop(true);
        return x.get_untracked();
    }

    let mut z = x.get_untracked(); // Get current state as mutable value
    z.extend(additional); // Extend with new images
    y(z); // Update state to new value
    x.get_untracked() // Return new list
    //logging::log!("{}, {} - {} | {}", old.len(), additional.len(), i, x.get_untracked().len());
}

//Images per infinite feed request
//(If this value is too high it seems to break infinite feed, around 10+ )
const FETCH_IMAGE_COUNT: usize = 10;

//Creates an infinite feed of images
#[component]
pub fn infinite_feed() -> impl IntoView {
    //Counter for what index to request form in DB
    let (stop, set_stop) = create_signal(false);
    let (db_index, set_db_index) = create_signal(0);
    let initImgs: Vec<ImageDb> = vec![];
    let (images, set_images) = create_signal(initImgs);
    //Signal with resource called every time the bottom is reached in infinite feed
    let _imageUpdater = create_resource(
        move || db_index.get(), 
        move |_| async move {
            append_imgs(set_images, images, fetch_files_and_handle_errors(
                        db_index.get_untracked(),
                        FETCH_IMAGE_COUNT).await, set_stop).await
        });

    let el = create_node_ref::<Div>();
    
    
    // //Creates and loads infinite feed
    let _ = use_infinite_scroll_with_options(
        el,
        move |_| async move {
            if stop.get_untracked(){
                return; // TODO:: Look into disabling the entire thing instead of just returning forever
            }
            logging::log!("{}", stop.get_untracked());
            //Index counter for DB
            let newIndex = db_index.get_untracked() + FETCH_IMAGE_COUNT; 
            logging::log!("Requesting {} more images. Index: {}", FETCH_IMAGE_COUNT, db_index.get_untracked()+FETCH_IMAGE_COUNT); 
            set_db_index.set(newIndex);
        },
        UseInfiniteScrollOptions::default().distance(10.0),
    );

    view! {
        <div class="flowdiv" node_ref=el> //class="flowdiv"
            <For each=move || images.get() key=|i| i.clone() let:item>
                <img src={format!("data:image/jpeg;base64,{}", item.path)} alt="Base64 Image" class="image imageSmooth" />
            </For> 
        </div>
    }
}

