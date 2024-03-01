use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
extern crate rand; 
use rand::Rng;
use leptos::html::Div;
use leptos_use::{UseInfiniteScrollOptions, use_infinite_scroll_with_options};
extern crate lazy_static;
use std::sync::Mutex;



#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/photo-album.css"/>

        // sets the document title
        <Title text="photo-album"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <TopBar/>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="test" view=TestPage/>
                    <Route path="hello" view=HelloPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[derive(
    Debug, Clone, leptos::server_fn::serde::Serialize, leptos::server_fn::serde::Deserialize,
)]
pub struct Folder {
    id: String,
    parent_id: Option<String>,
    name: String,
}

#[cfg(feature = "ssr")]
pub mod db;

#[server(TestDB, "/api")]
pub async fn test_db(name: String) -> Result<Vec<Folder>, ServerFnError> {
    use uuid::Uuid;

    // Connect to db
    let conn = crate::app::db::db().await?;

    // Insert & parameters example. Uncomment to add to DB.
    use rusqlite::params;
    let _ = conn.execute("INSERT INTO folder (id, name, createdDate) values (?1, ?2, ?3)", 
        params![Uuid::new_v4().to_string(),name, "now:)"])?;

    let mut stmnt = conn.prepare("SELECT id, parentId, name FROM folder")?;

    let folders = stmnt.query_map([], |row| {
        Ok(Folder {
            id: row.get(0)?,
            parent_id: row.get(1)?,
            name: row.get(2)?,
        })
    })?;

    let mut vec = Vec::new();
    for folder in folders {
        vec.push(folder.unwrap());
    }

    use std::cmp;
    Ok(vec[cmp::max(vec.len() - cmp::min(vec.len(), 10), 0)..].to_vec())
}

#[component]
pub fn TestDBButton() -> impl IntoView {
    let (name, _set_name) = create_signal("Controlled".to_string());
    let input_el: NodeRef<html::Input> = create_node_ref();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let value = input_el().expect("<Input> should be mounted").value();
        spawn_local(async {
            logging::log!("{:?}", test_db(value).await.unwrap());
        });
    };

    view! {
        <form on:submit=on_submit>
            <input type="text"
                value=name
                node_ref=input_el
            />
            <input type="submit" value="Submit"/>
        </form>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {

    view! {
        <TestDBButton></TestDBButton>
        <h1>"Home"</h1>
        // <DynamicList initial_length=5 initial_period=1/>
        <InfiniteFeed/>
    }
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
fn infinite_feed() -> impl IntoView {
    let (images, wImages) = create_signal(Vec::new());
    let (start, wStart) = create_signal(0);
    let el = create_node_ref::<Div>();

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

    //Initiate feed
    wImages.set(fetch_images(start.get(), 1)); 
    view! {
        <div
            class="flowdiv"
            node_ref=el
            >
            //Loop through all newly requested images
            <For each=move || images.get() key=|i| i.clone() let:image>
                {match image{
                    //Image
                    Element::Image(..) => view!{
                        <div class="image">
                            <img 
                            src={match image{
                                Element::Image(ref img) => img.src.to_string(),
                                _ => "".to_string()
                                } 
                            }
                            />
                        </div>
                    },
                    //Date
                    Element::String(ref date) => {
                        let date_clone = date.clone(); //Allow str to reach all the way in
                        view!{
                        <div class="break date_title">{
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



#[component]
fn TestPage() -> impl IntoView {
    let el = create_node_ref::<Div>();

    let (data, set_data) = create_signal(vec![1, 2, 3, 4, 5, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7]);

    let _ = use_infinite_scroll_with_options(
        el,
        move |_| async move {
            let len = data.with_untracked(|d| d.len());
            set_data.update(|data| *data = (1..len + 6).collect());
        },
        UseInfiniteScrollOptions::default().distance(10.0),
    );

    view! {
        <div
          class="flowdiv"
          node_ref=el
        >
            <ul>
            <For each=move || data.get() key=|i| *i let:item>
                <li>{item}</li>
            </For>
            </ul>
        </div>
    }
}

#[component]
fn HelloPage() -> impl IntoView {
    
    let (name, set_name) = create_signal("Potato".to_string());
    
    view! {
        <h1>"Hello"</h1>
        <input type="text"
        on:input=move |ev| {
            set_name(event_target_value(&ev));
        }
        prop:value=name
        />
        <p>"Name is: " {name}</p>
    }
}

#[component]
fn TopBar() -> impl IntoView {
    // All routes accessible from navigation bar
    view! {
        <nav>
            <a href="/">"Family Album"</a> // TODO Set to Admin defined name
            <a href="test">"Test"</a>
            <a href="hello">"Hello"</a>
        </nav>
    }
}
