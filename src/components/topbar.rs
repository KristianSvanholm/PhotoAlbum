use crate::auth::User;
use leptos::*;

#[component]
pub fn TopBar(_user: User) -> impl IntoView {
    /*

    use crate::auth::get_user;
    use crate::components::{signup::Signup, logout::Logout, login::Login};


    // All routes accessible from navigation bar
    view! {
        <nav>
            <a href="/">"Family Album"</a> // TODO Set to Admin defined name
            <a href="upload">"Upload"</a>
            <a href="settings">"Settings"</a>

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
                                    <span>
                                        <a href="settings">"Settings"</a>
                                        {format!("Logged in as: {} ({})", user.username, user.id)}
                                    </span>
                                }
                                    .into_view()
                            }
                        })
                }}

            </Transition>

        </nav>

    }
    */
}
