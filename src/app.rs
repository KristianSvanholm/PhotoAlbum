use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
extern crate rand; 
use rand::Rng;
use leptos::html::Div;
use leptos_use::{UseInfiniteScrollOptions, use_infinite_scroll_with_options};



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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Image {
    src: String,
    date: String,
}

const FETCH_IMAGE_COUNT: usize = 20;

fn fetch_images(start: usize, count: usize) -> Vec<Image> {
    let mut images = Vec::new();
    for i in start..start + count {
        let image = Image {
            src: format!("https://picsum.photos/200/300?random={}", i),
            date: 
                format!(
                    "{}-{:02}-{:02}",
                    rand::thread_rng().gen_range(2010..2022),
                    rand::thread_rng().gen_range(1..13),
                    rand::thread_rng().gen_range(1..29)
                ),
        };
        images.push(image);
    }
    images
}

#[component]
fn infinite_feed() -> impl IntoView {
    let (images, wImages) = create_signal(Vec::new());
    let (start, wStart) = create_signal(0);

    let el = create_node_ref::<Div>();

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

    wImages.set(fetch_images(start.get(), FETCH_IMAGE_COUNT));
    view! {
        <div
            class="flowdiv"
            node_ref=el
            style="display: flex; flex-wrap: wrap; gap: 10px;"
            >
            <For each=move || images.get() key=|i| i.clone() let:image>
                <div class="image">
                    <img 
                    src=image.src
                    style=format!("height: {}px; width: {}px;", rand::thread_rng().gen_range(250..300), rand::thread_rng().gen_range(250..300))
                    />
                    <p>{image.date}</p>
                </div>
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
