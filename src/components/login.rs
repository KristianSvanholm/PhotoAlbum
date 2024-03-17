use leptos::*;
use leptos_router::*;

#[server(Login, "/api")]
pub async fn login(
    username: String,
    password: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::auth::ssr::{Credentials, auth, update_session, make_session_long_term};

    let mut auth = auth()?;

    let res = auth.authenticate(Credentials{username: username, password: password}).await;

    match res {
        Ok(user) => {
            match user{
                Some(user) => {
                    auth.login(&user).await?;
                    if remember.is_some(){
                        make_session_long_term().await?;
                    }
                    update_session().await?;
                    Ok(())
                },
                None => {
                    Err(ServerFnError::ServerError(
                        "Password does not match or user does not exist.".to_string(),
                    ))
                },
            }            
        },
        Err(_err) => Err(ServerFnError::ServerError(
                "An error occured".to_string(),
        )),
    }
}

#[component]
pub fn Login(
    action: Action<Login, Result<(), ServerFnError>>,
) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <h1>"Log In"</h1>
            <label>
                "User ID:"
                <input
                    type="text"
                    placeholder="User ID"
                    maxlength="32"
                    name="username"
                    class="auth-input"
                />
            </label>
            <br/>
            <label>
                "Password:"
                <input type="password" placeholder="Password" name="password" class="auth-input"/>
            </label>
            <br/>
            <label>
                <input type="checkbox" name="remember" class="auth-input"/>
                "Remember me?"
            </label>
            <br/>
            <button type="submit" class="button">
                "Log In"
            </button>
        </ActionForm>
    }
}
