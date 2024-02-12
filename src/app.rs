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
        <DynamicList initial_length=5/>
    }
}

#[component]
fn DynamicList(
    /// The number of counters to begin with.
    initial_length: usize,
) -> impl IntoView {
    // This dynamic list will use the <For/> component.
    // <For/> is a keyed list. This means that each row
    // has a defined key. If the key does not change, the row
    // will not be re-rendered. When the list changes, only
    // the minimum number of changes will be made to the DOM.

    // `next_counter_id` will let us generate unique IDs
    // we do this by simply incrementing the ID by one
    // each time we create a counter
    let mut next_counter_id = initial_length;

    // we generate an initial list as in <StaticList/>
    // but this time we include the ID along with the signal
    let initial_counters = (0..initial_length)
        .map(|id| (id, create_signal(id + 1)))
        .collect::<Vec<_>>();

    // now we store that initial list in a signal
    // this way, we'll be able to modify the list over time,
    // adding and removing counters, and it will change reactively
    let (counters, set_counters) = create_signal(initial_counters);

    let add_counter = move |_| {
        // create a signal for the new counter
        let sig = create_signal(next_counter_id - 1);
        // add this counter to the list of counters
        set_counters.update(move |counters| {
            // since `.update()` gives us `&mut T`
            // we can just use normal Vec methods like `push`
            counters.push((next_counter_id, sig))
        });
        // increment the ID so it's always unique
        next_counter_id += 1;
    };

    

    view! {
        <div>
            <button on:click=add_counter>
                "Add Counter"
            </button>
            <ul>
                // The <For/> component is central here
                // This allows for efficient, key list rendering
                <For
                    // `each` takes any function that returns an iterator
                    // this should usually be a signal or derived signal
                    // if it's not reactive, just render a Vec<_> instead of <For/>
                    each=counters
                    // the key should be unique and stable for each row
                    // using an index is usually a bad idea, unless your list
                    // can only grow, because moving items around inside the list
                    // means their indices will change and they will all rerender
                    key=|counter| counter.0
                    // `children` receives each item from your `each` iterator
                    // and returns a view
                    children=move |(id, (count, set_count))| {
                        view! {
                            <li 
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
                                                // NOTE: in this example, we are creating the signals
                                                // in the scope of the parent. This means the memory used to
                                                // store them will not be reclaimed until the parent component
                                                // is unmounted. Here, we're removing the signal early (i.e, before
                                                // the DynamicList is unmounted), so we manually dispose of the signal
                                                // to avoid leaking memory.
                                                //
                                                // This is only necessary in an example with nested signals like this one.
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
                                
                            </li>
                        }
                    }
                />
            </ul>
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