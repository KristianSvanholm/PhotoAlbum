use leptos::*;
use rand::Rng;
extern crate lazy_static;
use std::sync::Mutex;
use leptos::html::Div;
use leptos_use::{UseInfiniteScrollOptions, use_infinite_scroll_with_options};
use serde::Serialize;
use serde::Deserialize;

//Image struct for images from DB
#[cfg_attr(feature="ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct ImageDb {
    id: String,
    path: String,
    uploadDate: String,
    createdDate: String,
}


//Fetch images from database
#[server(Feed, "/api")]
pub async fn fetch_files() -> Result<Vec<ImageDb>, ServerFnError> {
    //DB connection
    use crate::app::ssr::*;
    let pool = pool()?; 

    //Limit calls for later commit
    // LIMIT {} OFFSET {}",
    // count, start_index

    //Fetch images in descending order
    let files = sqlx::query_as::<_, ImageDb>(
        "SELECT id, path, uploadDate, createdDate FROM files ORDER BY createdDate DESC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(files)
}

//Image data struct
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Image {
    src: String,
    date: String,
}

//Takes a date string and image struct
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Element {
    String(String),
    Image(Image),
}

//Store previous fetched date from previous request to prevent duplicate date titles
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PreviousDate {
    month: String,
    year: String,
}

//Store last fetched date as global variable
lazy_static::lazy_static! {
    static ref PREVIOUS_DATE: Mutex<PreviousDate> = Mutex::new(PreviousDate{
        month: String::new(),
        year: String::new(),
    });
}

//Images per infinite feed request
const FETCH_IMAGE_COUNT: usize = 20;

//Fetch a number of images from database (currently random images from the web)
fn fetch_images(start: usize, count: usize) -> Vec<Element> {
    let mut images = Vec::new();
    //Generate images for infinite feed
    for i in start..start + count {
        let image = Image {
            src: format!("https://picsum.photos/{}/{}?random={}",rand::thread_rng().gen_range(200..500).to_string(),rand::thread_rng().gen_range(200..500).to_string(), i),
            date: 
                format!(
                    "{}-{:02}-{:02}",
                    rand::thread_rng().gen_range(2010..2023),
                    rand::thread_rng().gen_range(1..13),
                    rand::thread_rng().gen_range(1..29),
                ),
        };
        images.push(image);
    }
    images.sort_by_key(|image| image.date.clone());
    
    //New vector with months and years seperated
    let mut grouped_images: Vec<Element> = Vec::new();

    //Access previous date requested
    let mut previous_date = PREVIOUS_DATE.lock().unwrap();
    let mut current_month = previous_date.month.clone();
    let mut current_year = previous_date.year.clone();

    //Iterates over sorted images and adds years and months
    for image in images {
        let year = image.date[0..4].to_string();
        let month = image.date[5..7].to_string();
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
        grouped_images.push(Element::Image(image));
    }     
    previous_date.month = current_month;
    previous_date.year = current_year;

    grouped_images
}

//Creates an infinite feed of images
#[component]
pub fn infinite_feed() -> impl IntoView {
    let (images, wImages) = create_signal(Vec::new());
    let (start, wStart) = create_signal(0);
    let el = create_node_ref::<Div>();

    //Change feed display variables
    let (name, set_name) = create_signal("Smooth feed".to_string());
    let (feedDisplayClass, set_feedDisplayClass) = create_signal("break date_title".to_string());
    let (imageDisplayClass, set_imageDisplayClass) = create_signal("image".to_string());
    let (num, set_num) = create_signal(0);

    //Creates and loads infinite feed
    let _ = use_infinite_scroll_with_options(
        el,
        move |_| async move {
            let count = FETCH_IMAGE_COUNT; 
            let newStart = start.get() + count;
            let newImages = fetch_images(newStart, count);
            wImages.update(|images| images.extend(newImages));
            wStart.set(newStart);
        },
        UseInfiniteScrollOptions::default().distance(250.0),
    );

    //Fetch images from db and make async compatible
    let imgres = create_resource(move || (), move |_| fetch_files());

    //Initiate feed
    wImages.set(fetch_images(start.get(), 1)); 
    view! {

        <Transition fallback=move || {
            view! { <span>"Loading images..."</span> }
        }>
        {move || {
            imgres.get()
                .map(|imgres| match imgres {
                    Err(_) => {
                        view! {
                            <p>Failed to load image</p>
                        }
                            .into_view()
                    }
                    Ok(imgres) => {
                        view! {
                            <img src={format!("data:image/jpeg;base64,{}",imgres[0].path)} alt="Base64 Image"/>
                            <img src={format!("data:image/jpeg;base64,{}",imgres[1].path)} alt="Base64 Image"/>
                            <img src={format!("data:image/jpeg;base64,{}",imgres[2].path)} alt="Base64 Image"/>
                            <img src={format!("data:image/jpeg;base64,{}",imgres[3].path)} alt="Base64 Image"/>
                            <img src={format!("data:image/jpeg;base64,{}",imgres[4].path)} alt="Base64 Image"/>

                        }
                            .into_view()
                    }
                })
        }}
        </Transition>
       
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
        <div
            class="flowdiv"
            node_ref=el
            >
            //Loop through all newly requested images
            <For each=move || images.get() key=|i| i.clone() let:image>
                { match image{
                    //Image
                    Element::Image(ref img) => {
                        view!{
                        <div class={move || imageDisplayClass.get()}>
                            <img 
                            src={img.src.to_string()}
                            />
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

