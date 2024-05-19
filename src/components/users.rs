use leptos::*;

#[cfg(feature = "ssr")]
use sqlx::prelude::FromRow;

#[cfg(feature = "ssr")]
#[derive(Debug, FromRow)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

#[server(GetUserMap, "/api")]
pub async fn get_user_map() -> Result<std::collections::HashMap<i64, String>, ServerFnError> {
    use crate::db::ssr::pool;
    use std::collections::HashMap;

    let pool = pool()?;

    let users = sqlx::query_as::<_, UserInfo>("SELECT id, username FROM users")
        .fetch_all(&pool)
        .await?;

    let results: HashMap<_, _> = users.iter().map(|x| (x.id, x.username.clone())).collect();
    Ok(results)
}
