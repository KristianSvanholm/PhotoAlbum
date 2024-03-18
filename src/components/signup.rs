use leptos::*;
use leptos_router::*;

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct Invite {
    user_id: i64,
    username: String,
}

#[server(Signup, "/api")]
pub async fn signup(
    email: String,
    password: String,
    password_confirmation: String,
    remember: Option<String>,
    invite: String,
) -> Result<(), ServerFnError> {
    //TODO check if invitation is expired

    use bcrypt::{hash, DEFAULT_COST};
    use crate::db::ssr::*;
    use crate::auth::ssr::{SqlUser, auth};
    use crate::session::session_expiry::make_session_long_term;

    let pool = pool()?;
    let mut auth = auth()?;

    if password != password_confirmation {
        return Err(ServerFnError::ServerError(
            "Passwords did not match.".to_string(),
        ));
    }
    
    let invited_user = sqlx::query_as::<_, Invite>(
            "SELECT i.user_id, u.username 
            FROM invites i 
            INNER JOIN users u on u.id = i.user_id 
            WHERE token = ?"
        )
        .bind(&invite)
        .fetch_one(&pool)
        .await?;

    let password_hashed = hash(password, DEFAULT_COST).unwrap();

    sqlx::query("UPDATE users SET 
            email = ?,
            password = ?,
            signed_up = true
            WHERE id = ?"
        ).bind(email.clone())
        .bind(password_hashed)
        .bind(invited_user.user_id)
        .execute(&pool)
        .await?;

    sqlx::query("DELETE FROM invites 
        WHERE token = ?"
    ).bind(&invite)
    .execute(&pool)
    .await?;

    let user =
        SqlUser::get_from_username(invited_user.username, &pool)
            .await
            .ok_or_else(|| {
                ServerFnError::new("Signup failed: User does not exist.")
            })?;

    auth.login(&user).await?;
    if remember.is_some(){
        make_session_long_term().await?;
    }

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
