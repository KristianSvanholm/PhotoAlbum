use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use crate::components::{
    login::Login, 
    logout::Logout, 
    signup::Signup,
    //topbar::TopBar
};

#[component]
pub fn App() -> impl IntoView {
    use crate::auth::get_user;

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
                    <a href="admin">"Admin panel"</a>

                </nav>


                //###############


                <Routes>
                    // Route
                    <Route path="" view=HomePage/>
                    <Route path="upload" view=UploadPage/>
                    <Route path="signup/:invite" view=move || view! { <Signup action=signup/> }/>
                    <Route path="login" view=move || view! { <Login action=login/> }/>
                    <Route path="admin" view=AdminPanel/>
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

#[component]
fn AdminPanel() -> impl IntoView {
    // todo:: Probably rename to User Manager or something
    use crate::components::invite::InvitePanel;

    view! {
        <h1>Admin panel</h1>
        <InvitePanel/>
    }
}
