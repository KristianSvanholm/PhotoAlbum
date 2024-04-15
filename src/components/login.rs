use leptos::*;
use leptos_router::*;

#[server(Login, "/api")]
pub async fn login(
    username: String,
    password: String,
    remember: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::auth::ssr::{Credentials, auth};
    use crate::session::session_expiry::make_session_long_term;

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
        <ActionForm action=action class="loginForm">
            <h1>"Log in to the photo album:"</h1>

            <input
                type="text"
                placeholder="User ID"
                maxlength="32"
                name="username"
            />
            <br/>
            <input 
                type="password" 
                placeholder="Password" 
                name="password" 
            />
            <br/>
            <label class="rememberLabel">
                <input type="checkbox" name="remember" class="rememberCheckbox"/>
                "Remember me?"
            </label>
            <br/>
            <button type="submit" class="loginButton">
                "Log In"
            </button>
        </ActionForm>
    }
}
