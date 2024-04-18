use leptos::*;
use serde::{Deserialize, Serialize};
use leptos::html::Input;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;
#[cfg(feature = "ssr")]
use crate::auth;

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub signed_up: bool,
}

#[server(GetUserList, "/api")]
pub async fn get_user_list() -> Result<Vec<UserInfo>, ServerFnError> {
    let _admin = auth::authorized("admin").await?;

    use crate::db::ssr::pool;
    let pool = pool()?;
    let users =sqlx::query_as::<_, UserInfo>(
            "SELECT id, username, email, signed_up FROM users"
        ).fetch_all(&pool)
        .await?;

    Ok(users)
}


#[server(Invite, "/api")]
pub async fn invite(id: i64) -> Result<String, ServerFnError> {    
    use crate::auth::ssr::SqlUser;
    use crate::db::ssr::pool;

    // admin auth requirement
    // Get the current user
    // This will fail if the user is not logged in.
    let admin = auth::authorized("admin").await?;

    let pool = pool()?;

    // TODO: allow only one link per user at a time. 
    // This will fail if no such user exists and exit the request.
    let _user = sqlx::query_as::<_, SqlUser>(
            "SELECT * FROM users WHERE signed_up=false and id = ?"
        ).bind(id).fetch_one(&pool).await?;    

    let link = create_invitation_link(&id, &admin.id, &pool).await?;

    Ok(link) 
}

#[cfg(feature = "ssr")]
pub async fn create_invitation_link(
    user_id: &i64,
    admin_id: &i64,
    pool:&SqlitePool
)-> Result<String, ServerFnError>{

    use uuid::Uuid;
    let invite_token = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO invites (token, user_id, admin_id) VALUES (?,?,?)")
        .bind(invite_token.clone())
        .bind(user_id)
        .bind(admin_id)
        .execute(pool)
        .await?;

    let link = "/signup/".to_string()+&invite_token;
    
    Ok(link) 
}

#[server(CreateUser,"/api")]
pub async fn create_user(username: String) -> Result<(), ServerFnError>{
    // admin auth requirement
    let _admin = auth::authorized("admin").await?;
    use crate::db::ssr::pool;
    let pool = pool()?;

    sqlx::query("INSERT INTO users (username) VALUES (?)")
        .bind(username)
        .execute(&pool)
        .await?;

    Ok(())
}

#[server(MakeUserAdmin,"/api")]
pub async fn make_user_admin(id: i64) -> Result<(), ServerFnError>{
    // admin auth requirement
    let _admin = auth::authorized("admin").await?;

    use crate::db::ssr::pool;
    let pool = pool()?;

    sqlx::query("UPDATE users SET
            admin = true
            WHERE id = ?"
        ).bind(id)
        .execute(&pool)
        .await?;

    Ok(())
}


#[component]
pub fn InvitePanel() -> impl IntoView {
    let users = create_resource(|| (), |_| async { get_user_list().await });

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
                                            <Show when=move || !user.signed_up>
                                                <button on:click=move |_|{
                                                    spawn_local(async move {
                                                        let token = invite(user.id).await.unwrap(); 
                                                        let origin = window().location().origin().unwrap_or_default();
                                                        logging::log!("{}", token);
                                                        let url = origin+&token;
                                                        logging::log!("{}", url);
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
