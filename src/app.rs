use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

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


#[derive(Debug, Clone, leptos::server_fn::serde::Serialize, leptos::server_fn::serde::Deserialize)]
pub struct Folder {
    id: i32,
    parent_id: Option<i32>,
    name: String,
} 

#[cfg(feature = "ssr")]
pub mod db;

#[server(TestDB, "/api")]
pub async fn test_db(name: String) -> Result<Vec<Folder>, ServerFnError> {
    let conn = crate::app::db::db().await?;

     // Insert & parameters example. Uncomment to add to DB.
    use rusqlite::params;
    let _ = conn.execute(
            "INSERT INTO folder (name) values (?1)",
            params![name],
        )?; 
    

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
    Ok(vec[cmp::max(vec.len()-cmp::min(vec.len(),10),0)..].to_vec())
}

#[component]
pub fn TestDBButton() -> impl IntoView {

    let (name, set_name) = create_signal("Controlled".to_string());
    let input_el: NodeRef<html::Input> = create_node_ref();

    let on_submit =  move |ev: leptos::ev::SubmitEvent| {
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
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <TestDBButton></TestDBButton>
        <h1>"Home"</h1>
        <h1>HELLO WORLD</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}

#[component]
fn TestPage() -> impl IntoView {
    view! {
        <h1>"test"</h1>
    }
}

#[component]
fn HelloPage() -> impl IntoView {
    view! {
        <h1>"Hello"</h1>
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
