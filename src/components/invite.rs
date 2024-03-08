use leptos::*;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: String,
}

#[server(AllUninvited, "/api")]
pub async fn get_all_uninvited() -> Result<Vec<UserInfo>, ServerFnError> {

    use crate::db::ssr::pool;
    let pool = pool()?;
    let users =sqlx::query_as::<_, UserInfo>(
            "SELECT id, username, email FROM users WHERE invited=false"
        ).fetch_all(&pool)
        .await?;

    Ok(users)
}

/*
#[server(Invite, "/api")]
pub async fn invite(username: String) -> Result<(), ServerFnError> {
    

}
*/

#[component]
pub fn InvitePanel() -> impl IntoView {
    let users = create_resource(|| (), |_| async { get_all_uninvited().await });

    view! {
        <Suspense fallback=move || view! {<p>"Loading users"</p>}>
            <ErrorBoundary fallback=|_| {view! {<p>"Something went wrong"</p>}}>
                {move || {
                    users.get().map(move |x|{
                       x.map(move |y| {
                            view! {
                                <ul>
                                    {y.into_iter()
                                        .map(|user| view! {<li>{user.username}</li>})
                                        .collect_view()}
                                </ul>
                            }               
                       }) 
                    })

                    }
                }
            </ErrorBoundary>
        </Suspense>

    }
}
