use leptos::*;
use leptos::html::Div;
use leptos_use::{UseInfiniteScrollOptions, use_infinite_scroll_with_options};
use serde::Serialize;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::sync::Mutex;
extern crate lazy_static;

//Image struct for images from DB
#[cfg_attr(feature="ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct ImageDb {
    id: String,
    path: String,
    upload_date: String,
    created_date: String,
}

//Store previous fetched date from previous request to prevent duplicate date titles
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PreviousDate {
    month: String,
    year: String,
}

//Takes a date string and image struct
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum Element {
    String(String),
    ImageDb(ImageDb),
}

//Store last fetched date as global variable
lazy_static::lazy_static! {
    static ref PREVIOUS_DATE: Mutex<PreviousDate> = Mutex::new(PreviousDate{
        month: String::new(),
        year: String::new(),
    });
}

//Fetch images from database
#[server(Feed, "/api")]
pub async fn fetch_files(db_index: usize, count: usize) -> Result<Vec<ImageDb>, ServerFnError> {

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

    logging::log!("{} - {} - {}", db_index, count, files.len());
    Ok(files)
}

//Helper function to build the vector of images as they are requested from db
pub async fn fetch_files_and_handle_errors(db_index: usize, count: usize, ready: WriteSignal<bool>) -> Vec<Element> {
    logging::log!("please? {}", db_index);

    ready(false);
    match fetch_files(db_index, count).await {
        //Extend the current vector if ok
        Ok(files) => {
            logging::log!("got it! {}", db_index);
             //New vector with months and years seperated
            let mut grouped_images: Vec<Element> = Vec::new();

            //Access previous date requested
            let mut previous_date = PREVIOUS_DATE.lock().unwrap();
            let mut current_month = previous_date.month.clone();
            let mut current_year = previous_date.year.clone();
            
            let mut c: i64 = 0;
            //Iterates over sorted images and adds years and months
            for image in files {
                let year = image.created_date[0..4].to_string();
                let month = image.created_date[5..7].to_string();
                if month != current_month || year != current_year{
                    //Add year on change
                    if year != current_year{
                        grouped_images.push(Element::String(year.to_string()));
                        current_year = year.to_string();
                    }
                    //Add month on change
                    grouped_images.push(Element::String(month.to_string()));
                    current_month = month.to_string();
                }
                c=c+1;
                grouped_images.push(Element::ImageDb(image));
            }     
            previous_date.month = current_month;

            previous_date.year = current_year;

            logging::log!("total {}", c);

            if c != 0 {
                ready(true);
            }

            grouped_images
        }, 
        //Returns the incoming vector, e.g. if the there are no more images in db
        Err(_) => vec![],
    }
}

async fn append_imgs(
        y: WriteSignal<Vec<Element>>, 
        x: ReadSignal<Vec<Element>>, 
        additional: Vec<Element>, 
) -> Vec<Element> {
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

    let (ready, set_ready) = create_signal(true);
    
    let (db_index, set_db_index) = create_signal(0);
    
    let initImgs: Vec<Element> = vec![];
    let (images, set_images) = create_signal(initImgs);
    
    //Signal with resource called every time the bottom is reached in infinite feed
    let _imageUpdater = create_resource(
        move || db_index.get(), 
        move |_| async move {
            append_imgs(set_images, images, fetch_files_and_handle_errors(
                        db_index.get_untracked(),
                        FETCH_IMAGE_COUNT, set_ready).await).await
        });

    let el = create_node_ref::<Div>();

    //Change feed display variables (smooth/date)
    let (name, set_name) = create_signal("Smooth feed".to_string());
    let (feedDisplayClass, set_feedDisplayClass) = create_signal("break date_title".to_string());
    let (imageDisplayClass, set_imageDisplayClass) = create_signal("image".to_string());
    let (num, set_num) = create_signal(0);
    
    // //Creates and loads infinite feed
    let _ = use_infinite_scroll_with_options(
        el,
        move |_| async move {
            if !ready.get_untracked(){
                return; // TODO:: Look into disabling the entire thing instead of just returning forever
            }

            //Index counter for DB
            let newIndex = db_index.get_untracked() + FETCH_IMAGE_COUNT; 
            logging::log!("Requesting {} more images. Index: {}", FETCH_IMAGE_COUNT, db_index.get_untracked()+FETCH_IMAGE_COUNT);
            set_db_index.set(newIndex);
        },
        UseInfiniteScrollOptions::default().distance(10.0),
    );

    view! {
        //Change display of feed
        <button on:click=move |_| {
            if num.get() == 0 {
                set_name("Date feed".to_string());
                set_feedDisplayClass("invis".to_string());
                set_imageDisplayClass("image imageSmooth".to_string());
                set_num(1);
            } else {
                set_name("Smooth feed".to_string());
                set_feedDisplayClass("break date_title".to_string());
                set_imageDisplayClass("image".to_string());
                set_num(0);
            }
            }>{name}</button>
        <div class="flowdiv" node_ref=el> //class="flowdiv"
            <For each=move || images.get() key=|i| i.clone() let:item>

            { match item{
                //Image
                Element::ImageDb(ref img) => {
                    view!{
                    <div class={move || imageDisplayClass.get()}>
                    <img src={format!("data:image/jpeg;base64,{}", img.path)} alt="Base64 Image" class="image imageSmooth" />
                    </div>
                }},
                //Date
                Element::String(ref date) => {
                    let date_clone = date.clone(); //Allow str to reach all the way in
                    view!{
                    <div class={move || feedDisplayClass.get()}>{
                        match date_clone.parse().unwrap() {
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
                            _ => date_clone
                        }
                    }</div>
                }}  
            }}
            </For> 
        </div>
    }
}

