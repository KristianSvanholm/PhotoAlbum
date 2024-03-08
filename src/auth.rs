use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub permissions: HashSet<String>,
}

impl Default for User {
    fn default() -> Self {
        let permissions = HashSet::new();

        Self {
            id: -1,
            username: "Guest".into(),
            password: "".into(),
            email: "".into(),
            permissions,
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    pub use super::User;
    pub use axum_session_auth::{
        Authentication, HasPermission, SessionSqlitePool,
    };
    pub use sqlx::SqlitePool;
    pub use std::collections::HashSet;
    pub type AuthSession = axum_session_auth::AuthSession<
        User,
        i64,
        SessionSqlitePool,
        SqlitePool,
    >;
    pub use crate::app::ssr::{auth, pool};
    pub use async_trait::async_trait;
    pub use bcrypt::{hash, verify, DEFAULT_COST};

    impl User {
        pub async fn get(id: i64, pool: &SqlitePool) -> Option<Self> {
            let sqluser = sqlx::query_as::<_, SqlUser>(
                "SELECT * FROM users WHERE id = ?",
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .ok()?;

            //lets just get all the tokens the user can use, we will only use the full permissions if modifying them.
            let sql_user_perms = sqlx::query_as::<_, SqlPermissionTokens>(
                "SELECT token FROM user_permissions WHERE user_id = ?;",
            )
            .bind(id)
            .fetch_all(pool)
            .await
            .ok()?;

            Some(sqluser.into_user(Some(sql_user_perms)))
        }

        pub async fn get_from_username(
            name: String,
            pool: &SqlitePool,
        ) -> Option<Self> {
            let sqluser = sqlx::query_as::<_, SqlUser>(
                "SELECT * FROM users WHERE username = ?",
            )
            .bind(name)
            .fetch_one(pool)
            .await
            .ok()?;

            //lets just get all the tokens the user can use, we will only use the full permissions if modifying them.
            let sql_user_perms = sqlx::query_as::<_, SqlPermissionTokens>(
                "SELECT token FROM user_permissions WHERE user_id = ?;",
            )
            .bind(sqluser.id)
            .fetch_all(pool)
            .await
            .ok()?;

            Some(sqluser.into_user(Some(sql_user_perms)))
        }
    }

    #[derive(sqlx::FromRow, Clone)]
    pub struct SqlPermissionTokens {
        pub token: String,
    }

    #[async_trait]
    impl Authentication<User, i64, SqlitePool> for User {
        async fn load_user(
            userid: i64,
            pool: Option<&SqlitePool>,
        ) -> Result<User, anyhow::Error> {
            let pool = pool.unwrap();

            User::get(userid, pool)
                .await
                .ok_or_else(|| anyhow::anyhow!("Cannot get user"))
        }

        fn is_authenticated(&self) -> bool {
            true
        }

        fn is_active(&self) -> bool {
            true
        }

        fn is_anonymous(&self) -> bool {
            false
        }
    }

    #[async_trait]
    impl HasPermission<SqlitePool> for User {
        async fn has(&self, perm: &str, _pool: &Option<&SqlitePool>) -> bool {
            self.permissions.contains(perm)
        }
    }

    #[derive(sqlx::FromRow, Clone)]
    pub struct SqlUser {
        pub id: i64,
        pub username: String,
        pub email: String,
        pub password: String,
    }

    impl SqlUser {
        pub fn into_user(
            self,
            sql_user_perms: Option<Vec<SqlPermissionTokens>>,
        ) -> User {
            User {
                id: self.id,
                username: self.username,
                email: self.email,
                password: self.password,
                permissions: if let Some(user_perms) = sql_user_perms {
                    user_perms
                        .into_iter()
                        .map(|x| x.token)
                        .collect::<HashSet<String>>()
                } else {
                    HashSet::<String>::new()
                },
            }
        }
    }
}

#[server]
pub async fn get_user() -> Result<Option<User>, ServerFnError> {
    use crate::app::ssr::auth;

    let auth = auth()?;

    Ok(auth.current_user)
}
