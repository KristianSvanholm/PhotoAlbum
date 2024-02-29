use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    use crate::components::topbar::TopBar;

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
                    <Route path="upload" view=UploadPage/>
                </Routes>
            </main>
        </Router>
    }
}

// ===== ONLY ROUTES ======
#[component]
fn HomePage() -> impl IntoView {
    use crate::components::testdb::TestDBButton;
    use crate::components::feed::InfiniteFeed;

    view! {
        <h1>"Home"</h1>
        <TestDBButton></TestDBButton>
        // <DynamicList initial_length=5 initial_period=1/>
        <InfiniteFeed/>

    }
}

#[component]
fn UploadPage() -> impl IntoView {

    use crate::components::upload::UploadMedia;

    view! {
        <h1>Upload</h1>
        <UploadMedia></UploadMedia>
    }
}
