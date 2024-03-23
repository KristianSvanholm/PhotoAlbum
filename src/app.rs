use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use crate::components::{
    login::Login, 
    logout::Logout, 
    signup::Signup,
    //topbar::TopBar
};

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::auth::ssr::AuthSession;
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
    use crate::auth::get_user;

    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();
    let signup = create_server_action::<Signup>();

    let user = create_local_resource(
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
                //###############

                <nav>
                    <Transition fallback=move || {
                        view! { <span>"Loading..."</span> }
                    }>
                        {move || {
                            user.get()
                                .map(|user| match user {
                                    Err(e) => {
                                        view! {
                                            <span>{format!("Login error: {}", e)}</span>
                                        }
                                            .into_view()
                                    }
                                    Ok(None) => {
                                        view! {}
                                            .into_view()
                                    }
                                    Ok(Some(user)) => {
                                        view! {
                                            <a href="/">"Home"</a>
                                            <a href="/upload">"Upload"</a>
                                            <a href="/settings">"Settings"</a>
                                            <a href="/admin">"Admin panel"</a>
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
                <main>
                //###############


                <Routes>
                    // Route
                    <Route path="/" view=move || {
                        view! {
                            <Show 
                                when=move || {user.get().map(|user| match user {
                                    Ok(Some(_)) => true,
                                    Ok(None) => false,
                                    Err(_) => false,
                                }).unwrap_or(false)} 
                                fallback= move || view! { <Login action=login/> }>
                                <Outlet/>
                            </Show>
                        }
                    }>
                        <Route path="/" view=HomePage/>
                        <Route path="/settings" view=move || {
                            view! {
                                <Logout action=logout/>
                            }
                        }/>
                        <Route path="/upload" view=UploadPage/>
                        <Route path="/admin" view=AdminPanel/>
                    </Route>
                    <Route path="/signup" view=move || {
                        view! {
                            <Show 
                                when=move || {user.get().map(|user| match user {
                                    Ok(Some(_)) => false,
                                    Ok(None) => true,
                                    Err(_) => true,
                                }).unwrap_or(true)}>
                                <Outlet/>
                            </Show>
                        }
                    }>
                        <Route path=":invite" view=move || view! { <Signup action=signup/> }/>
                    </Route>

                    <Route path="*any" view=move || view! { <h1>"Not Found"</h1> }/>
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

#[component]
fn AdminPanel() -> impl IntoView {
    // todo:: Probably rename to User Manager or something
    use crate::components::invite::InvitePanel;

    view! {
        <h1>Admin panel</h1>
        <InvitePanel/>
    }
}
