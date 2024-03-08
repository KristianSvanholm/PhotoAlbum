use leptos::*;
use leptos_router::*;

#[server(Signup, "/api")]
pub async fn signup(
    username: String,
    email: String,
    password: String,
    password_confirmation: String,
    remember: Option<String>,
    invite: String,
) -> Result<(), ServerFnError> {
    use bcrypt::{hash, DEFAULT_COST};
    use crate::db::ssr::*;
    use crate::auth::{User, ssr::auth};

    let pool = pool()?;
    let auth = auth()?;

    if password != password_confirmation {
        return Err(ServerFnError::ServerError(
            "Passwords did not match.".to_string(),
        ));
    }

    let password_hashed = hash(password, DEFAULT_COST).unwrap();

    sqlx::query("INSERT INTO users (username, email, password) VALUES (?,?,?)")
        .bind(username.clone())
        .bind(email.clone())
        .bind(password_hashed)
        .execute(&pool)
        .await?;

    let user =
        User::get_from_username(username, &pool)
            .await
            .ok_or_else(|| {
                ServerFnError::new("Signup failed: User does not exist.")
            })?;

    auth.login_user(user.id);
    auth.remember_user(remember.is_some());

    leptos_axum::redirect("/");

    Ok(())
}

#[component]
pub fn Signup(
    action: Action<Signup, Result<(), ServerFnError>>,
) -> impl IntoView {
    // Get invite from URL
    let params = use_params_map();
    let invite = params.with(|p| p.get("invite").cloned().unwrap_or_default());

    view! {
        <ActionForm action=action>
            <h1>"Sign Up"</h1>
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
                "Email:"
                <input
                    type="text"
                    placeholder="Email"
                    name="email"
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
                "Confirm Password:"
                <input
                    type="password"
                    placeholder="Password again"
                    name="password_confirmation"
                    class="auth-input"
                />
            </label>
            <br/>
            <label>
                "Remember me?" <input type="checkbox" name="remember" class="auth-input"/>
            </label>

            // Add invite string to request as hidden input element
            <input hidden name="invite" prop:value=invite/>

            <br/>
            <button type="submit" class="button">
                "Sign Up"
            </button>
        </ActionForm>
    }
}
