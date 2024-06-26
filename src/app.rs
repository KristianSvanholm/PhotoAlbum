use crate::components::{
    login::Login,
    logout::Logout,
    signup::Signup,
    //topbar::TopBar
};
use leptonic::{components::root::Root, components::theme::LeptonicTheme};
use leptos::{
    html::{Nav, ToHtmlElement},
    *,
};
use leptos_meta::*;
use leptos_router::*;
use leptos_use::use_event_listener;

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
        use_context::<AuthSession>()
            .ok_or_else(|| ServerFnError::ServerError("Auth session missing.".into()))
    }
}

#[component]
pub fn App() -> impl IntoView {
    use crate::auth::get_user;
    use leptos::ev::click;

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
        |_| async move {
            let user = get_user().await;
            if let Ok(Some(u)) = user.clone() {
                provide_context(u);
            }
            user
        },
    );

    let navref: leptos::NodeRef<Nav> = create_node_ref();

    let _ = use_event_listener(navref, click, move |ev| {
        let target = event_target::<web_sys::HtmlAnchorElement>(&ev).to_leptos_element();
        if Some(target.tag_name()) != Some("A".to_string()) {
            return;
        }
        let _ = target.class_list().add_1("active");
        let nav = navref.get_untracked().unwrap().children();
        for i in 0..nav.length() {
            let link = nav.get_with_index(i).unwrap().to_leptos_element();
            if Some(link.text_content()) != Some(target.text_content()) {
                let _ = link.class_list().remove_1("active");
            }
        }
    });

    provide_meta_context();

    view! {
        <Root default_theme=LeptonicTheme::default()>

        <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/5.15.4/css/all.min.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Stylesheet id="leptos" href="/pkg/photo-album.css"/>
        <Router>
                //###############
                <nav class="topbarNav" node_ref=navref>
                    <Transition fallback=move || {
                        view! { <span id="loading">"Loading..."</span> }
                    }>
                        {logging::log!("Requesting {:?}", user.get());}
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
                                        let c_user = user.clone();
                                        view! {
                                            <a href="/" class="active">"Home"</a>
                                            <Show when=move || {c_user.has("admin")}>
                                                <a href="/admin">"Admin"</a>
                                            </Show>
                                            <ActionForm action=logout class="topbarNav-right">
                                                <button type="submit">"Sign Out"</button>
                                                <span>
                                                {format!("Logged in as: {}({})", user.username, user.id)}
                                                </span>
                                            </ActionForm>
                                        }.into_view()
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
                                when=move||match user.get(){
                                    Some(_) => true,
                                    None => false
                                }
                                fallback= || view! {  <Loading/> }>
                                <Outlet/>
                            </Show>
                        }
                    }>
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
                            <Route path="/admin" view=move || {
                                view! {
                                    <Show
                                        when=move || {user.get().map(|user| match user {
                                            Ok(Some(user)) => user.has("admin"),
                                            _ => false,
                                        }).unwrap_or(false)}>
                                        <AdminPanel/>
                                    </Show>
                                }
                            }/>
                        </Route>
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
        </Root>
    }
}

// ===== ONLY ROUTES ======
#[component]
fn HomePage() -> impl IntoView {
    use crate::components::home_page::HomePage;

    view! {
        <HomePage/>
    }
}

#[component]
fn Loading() -> impl IntoView {
    use crate::components::loading::Loading_Triangle;

    view! {
        <div class="center-h" style="width:100%">
            <Loading_Triangle show=move || true/>
        </div>
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
