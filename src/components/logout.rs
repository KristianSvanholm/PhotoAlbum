use leptos::*;
use leptos_router::*;

#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    use crate::auth::ssr::auth;

    let auth = auth()?;

    auth.logout_user();
    leptos_axum::redirect("/");

    Ok(())
}

#[component]
pub fn Logout(
    action: Action<Logout, Result<(), ServerFnError>>,
) -> impl IntoView {
    view! {
        <div id="loginbox">
            <ActionForm action=action>
                <button type="submit" class="button">
                    "Log Out"
                </button>
            </ActionForm>
        </div>
    }
}
