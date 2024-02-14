use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
extern crate rand; 
use rand::Rng;

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

// div picture component
#[component]
fn PictureDiv() -> impl IntoView {
    view! {
        <div/>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {

    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Home"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
        <DynamicList initial_length=5 initial_period=1/>
    }
}

#[component]
fn DynamicList(
    /// The number of counters to begin with.
    initial_length: usize,
    initial_period: usize,
) -> impl IntoView {
    let mut next_counter_id = initial_length;
    let mut next_date_id = initial_period;

    let initial_counters = (0..initial_length)
        .map(|id| (id, create_signal(id + 1)))
        .collect::<Vec<_>>();

    let initial_date = (0..initial_period)
        .map(|id| (id, create_signal(id + 1)))
        .collect::<Vec<_>>();

    let (counters, set_counters) = create_signal(initial_counters);
    let (date, set_date) = create_signal(initial_date);


    let add_counter = move |_| {
        // create a signal for the new counter
        let sig = create_signal(next_counter_id - 1);
        // add this counter to the list of counters
        set_counters.update(move |counters| {
            counters.push((next_counter_id, sig))
        });
        next_counter_id += 1;
    };
    let add_date = move |_| {
        // create a signal for the new counter
        let sig = create_signal(next_date_id);
        // add this counter to the list of counters
        set_date.update(move |date| {
            date.push((next_date_id, sig))
        });
        next_date_id += 1;
    };

    

    view! {
        <div>
            <button on:click=add_counter>
                "Add image"
            </button>
            <button on:click=add_date>
            "Add date"
            </button>
            <h2>Date</h2>
            <For
            each=date
            key=|counter| counter.0
            children=move |(count, _set_count)| {
                view! {
                    <h2>April {count}</h2>
                    <div class="image-container">
                <For
                    each=counters
                    key=|counter| counter.0
                    children=move |(id, (count, set_count))| {
                        view! {
                            // <Show
                            // when=move || { count() % rand::thread_rng().gen_range(1..20) == 0 }
                            // >
                            // <h2>1st of April</h2>
                            // </Show>
                            <div class="image-div" 
                                style:width=rand::thread_rng().gen_range(150..350).to_string()+"px"
                                style:height=rand::thread_rng().gen_range(150..350).to_string()+"px"
                            > 
                                <button
                                    on:click=move |_| set_count.update(|n| *n += 1)
                                >
                                    {count}
                                </button>
                                <button
                                    on:click=move |_| {
                                        set_counters.update(|counters| {
                                            counters.retain(|(counter_id, (signal, _))| {
                                                if counter_id == &id {
                                                    signal.dispose();
                                                }
                                                counter_id != &id
                                            })
                                        });
                                    }
                                >
                                    "Remove"
                                </button>
                            </div>
                        }
                    }
                />
            </div>
                }
            }
            />
            
        </div>
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