use crate::auth::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use crate::components::{
    login::Login, 
    logout::Logout, 
    signup::Signup,
    //topbar::TopBar
};

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::auth::{ssr::AuthSession, User};
    use leptos::*;
    use sqlx::SqlitePool;

    pub fn pool() -> Result<SqlitePool, ServerFnError> {
        use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
    }

    pub fn auth() -> Result<AuthSession, ServerFnError> {
        use_context::<AuthSession>().ok_or_else(|| {
            ServerFnError::ServerError("Auth session missing.".into())
        })
    }
}

#[component]
pub fn App() -> impl IntoView {

    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();
    let signup = create_server_action::<Signup>();

    let user = create_resource(
        move || {
            (
                login.version().get(),
                signup.version().get(),
                logout.version().get(),
            )
        },
        move |_| get_user(),
    );
    provide_meta_context();

    view! {
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Stylesheet id="leptos" href="/pkg/photo-album.css"/>
        <Router>
            <main>
                
                //###############

                <nav>
                    <a href="/">"Family Album"</a> // TODO Set to Admin defined name
                    <a href="upload">"Upload"</a>

                    <Transition fallback=move || {
                        view! { <span>"Loading..."</span> }
                    }>
                        {move || {
                            user.get()
                                .map(|user| match user {
                                    Err(e) => {
                                        view! {
                                            <a href="login">"Login"</a>
                                            <a href="signup">"Signup"</a>
                                            <span>{format!("Login error: {}", e)}</span>
                                        }
                                            .into_view()
                                    }
                                    Ok(None) => {
                                        view! {
                                            <a href="login">"Login"</a>
                                            <a href="signup">"Signup"</a>
                                            <span>"Logged out."</span>
                                        }
                                            .into_view()
                                    }
                                    Ok(Some(user)) => {
                                        view! {
                                            <a href="settings">"Settings"</a>
                                            <span>
                                                {format!("Logged in as: {} ({})", user.username, user.id)}
                                            </span>
                                        }
                                            .into_view()
                                    }
                                })
                        }}

                    </Transition>

                </nav>


                //###############


                <Routes>
                    // Route
                    <Route path="" view=HomePage/>
                    <Route path="upload" view=UploadPage/>
                    <Route path="signup" view=move || view! { <Signup action=signup/> }/>
                    <Route path="login" view=move || view! { <Login action=login/> }/>
                    <Route
                        path="settings"
                        view=move || {
                            view! {
                                <h1>"Settings"</h1>
                                <Logout action=logout/>
                            }
                        }
                    />

                </Routes>
            </main>
        </Router>
    }
}

// ===== ONLY ROUTES ======
#[component]
fn HomePage() -> impl IntoView {
    use crate::components::feed::InfiniteFeed;

    view! {
        <h1>"Home"</h1>
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