#[cfg(feature = "ssr")]
use crate::auth;
use leptos::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::prelude::FromRow;

#[cfg_attr(feature = "ssr", derive(FromRow))]
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

#[server(GetUserList, "/api")]
pub async fn get_user_list_sans_admin() -> Result<Vec<UserInfo>, ServerFnError> {
    auth::logged_in().await?;
    use crate::db::ssr::pool;

    let pool = pool()?;

    let users =
        sqlx::query_as::<_, UserInfo>("SELECT id, username FROM users WHERE username != 'admin'")
            .fetch_all(&pool)
            .await?;

    Ok(users)
}
