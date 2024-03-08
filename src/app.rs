use crate::auth::*;
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
            //<Transition></Transition>
            <main>
                
                //###############

                <nav>
                    <a href="home">"Family Album"</a> // TODO Set to Admin defined name
                    <a href="upload">"Upload"</a>
                    <a href="settings"> "Settings"</a>

                    <Transition fallback=move || {
                        view! { <span>"Loading..."</span> }
                    }>
                        {move || {
                            user.get()
                                .map(|user| match user {
                                    Err(e) => {
                                        view! { <span>{format!("Login error: {}", e)}</span> }.into_view()
                                    }
                                    Ok(None) => {
                                        view! { <span>"Logged out."</span> }.into_view()
                                    }
                                    Ok(Some(user)) => {
                                        view! { <span> {format!("Logged in as: {} ({})", user.username, user.id)} </span> }.into_view()
                                    }
                                })
                        }}

                    </Transition>
                </nav>
                //<Transition>/* logged in as ... */</Transition>
                    //<Routes>
                        //<Route path="" view=move || match user.get() { Some(_) => view! {<Homepage/>}.into_view(), None => view! { <Login action=login/> }.into_view() }>
                        //<Route path="" view=HomePage />
                        //<Route path="upload" view=UploadPage />
                        /* other logged-in-only routes */
                        //<ProtectedRoute path="invite/:token" redirect_path="/" condition=move || user.get().is_none() view=move || view! { <Signup action=signup /> } />
                    //</Routes>

                //###############
                <Routes>
                    //<Route path="/" view=move || user.get().map(|user| match user { Ok(Some(_)) => view! {<Outlet/>}.into_view(), _ => view! { <Login action=login/> }.into_view() })>
                    <Route path="/" view=|| view! {
                        <Show when=|| !user.get().is_none() fallback=|| view! { <p>"Loading..."</p>}>
                            {
                                view! {<Outlet/>}
                            }
                        </Show>
                    }>
                        <Route path="/home" view=HomePage />
                        <Route path="upload" view=UploadPage />
                        <Route path="settings" view=move || view! {<Logout action=logout/>}/>
                        /* other logged-in-only routes */
                    </Route>
                    <ProtectedRoute ssr=SsrMode::Async path="invite/:token" redirect_path="/" condition=move || user.get().is_none() view=move || view! { <Signup action=signup /> } />
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
