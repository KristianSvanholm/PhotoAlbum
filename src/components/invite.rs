use leptos::*;
use serde::{Deserialize, Serialize};
use leptos::html::Input;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub invited: bool,
}

#[server(AllUninvited, "/api")]
pub async fn get_all_users() -> Result<Vec<UserInfo>, ServerFnError> {

    use crate::db::ssr::pool;
    let pool = pool()?;
    let users =sqlx::query_as::<_, UserInfo>(
            "SELECT id, username, email, invited FROM users"
        ).fetch_all(&pool)
        .await?;

    Ok(users)
}


#[server(Invite, "/api")]
pub async fn invite(id: i64) -> Result<String, ServerFnError> {
    
    use crate::auth::ssr::SqlUser;
    use crate::db::ssr::pool;
    use uuid::Uuid;

    // TODO:: Add admin auth requirement here.

    let pool = pool()?;

    // This will fail if no such user exists and exit the request.
    let _user = sqlx::query_as::<_, SqlUser>(
            "SELECT * FROM users WHERE invited=false and id = ?"
        ).bind(id).fetch_one(&pool).await?;
    
    let invite_token = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO invites (token, user_id, admin) VALUES (?,?,?)")
        .bind(invite_token.clone())
        .bind(id)
        .bind(false)
        .execute(&pool)
        .await?;

    Ok(invite_token) 
}

#[server(CreateUser,"/api")]
pub async fn create_user(username: String) -> Result<(), ServerFnError>{
    use crate::db::ssr::pool;
    let pool = pool()?;

    sqlx::query("INSERT INTO users (username, admin) VALUES (?,?)")
        .bind(username)
        .bind(false)
        .execute(&pool)
        .await?;

    Ok(())
}


#[component]
pub fn InvitePanel() -> impl IntoView {
    let users = create_resource(|| (), |_| async { get_all_users().await });

    let new_user_input = create_node_ref::<Input>();
    
    let create_user = move |_| { spawn_local(async move {
        let node = new_user_input.get_untracked().expect("create user should be loaded by now.");
        create_user(node.value()).await.unwrap();
        })
    };

    view! {
        <Suspense fallback=move || view! {<p>"Loading users"</p>}>
            <ErrorBoundary fallback=|_| {view! {<p>"Something went wrong"</p>}}>
                {move || {
                    users.get().map(move |x|{
                       x.map(move |y| {
                            view! {
                                <ul>
                                    {y.into_iter()
                                        .map(|user| view! {
                                            <li>{user.username}</li>
                                            <Show when=move || !user.invited>
                                                <button on:click=move |_|{
                                                    spawn_local(async move {
                                                        let token = invite(user.id).await.unwrap(); 
                                                        logging::log!("{}", token);
                                                })}> "invite" </button>
                                            </Show>})
                                        .collect_view()}
                                </ul>
                                <input 
                                    type="text"
                                    placeholder="New users name"
                                    name="username"
                                    class="auth-input"
                                    _ref=new_user_input
                                />
                                <button on:click=create_user>"Create new user"</button>
                            }               
                        }) 
                    })
                }
            }
            </ErrorBoundary>
        </Suspense>
    }
}
