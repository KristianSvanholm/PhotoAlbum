use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub permissions: HashSet<String>,
}

impl Default for User {
    fn default() -> Self {
        let permissions = HashSet::new();

        Self {
            id: -1,
            username: "Guest".into(),
            email: "".into(),
            permissions,
        }
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    pub use super::User;
    use axum_login::{AuthUser, AuthnBackend, UserId};
    use sqlx::SqlitePool;
    pub use std::collections::HashSet;
    pub type AuthSession = axum_login::AuthSession<
        Backend,
    >;
    pub use axum_login::tower_sessions::{Session, SessionManagerLayer, Expiry};
    pub use async_trait::async_trait;
    pub use bcrypt::{hash, verify, DEFAULT_COST};
    use serde::Deserialize;
    use time::{OffsetDateTime, Duration};
    use tokio::task;

    use leptos::*;

    pub fn auth() -> Result<AuthSession, ServerFnError> {
        use_context::<AuthSession>().ok_or_else(|| {
            ServerFnError::ServerError("Auth session missing.".into())
        })
    }

    pub fn session() -> Result<Session, ServerFnError> {
        use_context::<Session>().ok_or_else(|| {
            ServerFnError::ServerError("Session missing.".into())
        })
    }

    pub fn expiry_config() -> Result<CustomExpirySessionConfig, ServerFnError> {
        use_context::<CustomExpirySessionConfig>().ok_or_else(|| {
            ServerFnError::ServerError("Expiry config missing.".into())
        })
    }

    pub async fn update_session() -> Result<(), ServerFnError> {
        let session = session()?;
        let expiry_config = expiry_config()?;
        println!("Expiry: {}", session.expiry_date());
        let last_accessed: OffsetDateTime = session.get("last_accessed").await.unwrap().unwrap_or_else(|| OffsetDateTime::now_utc().saturating_sub(Duration::hours(2)));
        println!("Threshold: {}", OffsetDateTime::now_utc().saturating_add(expiry_config.expiry));
        //if last_accessed < OffsetDateTime::now_utc().saturating_sub(expiry_config.on_activity_check){
            session.insert("last_accessed", OffsetDateTime::now_utc()).await.unwrap();
            session.cycle_id().await?;
            println!("Updated");
            if session.expiry_date() < OffsetDateTime::now_utc().saturating_add(expiry_config.expiry){
                session.set_expiry(Some(Expiry::OnInactivity(expiry_config.expiry)));
                println!("Changed expiry");
            }
        //}
        println!("Last accessed: {}", last_accessed);
        
        Ok(())
    }

    pub async fn make_session_long_term() -> Result<(), ServerFnError> {
        let session = session()?;
        let expiry_config = expiry_config()?;
        let expired_at = 
            OffsetDateTime::now_utc()
            .saturating_add(expiry_config.max_age_term_expiry);
        println!("Long session: {}", expired_at);
        session.set_expiry(Some(Expiry::AtDateTime(expired_at)));
        session.insert("last_accessed", OffsetDateTime::now_utc()).await.unwrap();
        assert_eq!(session.expiry(), Some(Expiry::AtDateTime(expired_at)));
        println!("Long session: {}", session.expiry_date());
        Ok(())
    }

    #[derive(Debug, Clone, Copy)]
    pub struct CustomExpirySessionConfig{
        /// Expire on inactivity after expiry on default
        pub expiry: Duration,
        /// if remember me option is on allow long long sessions
        /// Switch to on inactivity after max_age_term_expiry
        pub max_age_term_expiry: Duration,
        /// time interval to check for to update expiry to prevent
        /// too many database writes
        /// needs to be smaller than expiry
        pub on_activity_check: Duration, 
    }

    impl Default for CustomExpirySessionConfig {
        fn default() -> Self {
            Self {
                expiry: Duration::hours(3),
                max_age_term_expiry: Duration::days(60), 
                on_activity_check: Duration::hours(1),
            }
        }
    }

    impl User {
        pub async fn has(&self, perm: &str) -> bool {
            self.permissions.contains(perm)
        }
    }

    impl AuthUser for SqlUser {
        type Id = i64;
    
        fn id(&self) -> Self::Id {
            self.id
        }
    
        fn session_auth_hash(&self) -> &[u8] {
            self.password.as_bytes()
        }
    }

    #[derive(Debug, Clone)]
    pub struct Backend {
        pool: SqlitePool,
    }

    impl Backend {
        pub fn new(pool: SqlitePool) -> Self {
            Self {pool}
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct Credentials {
        pub username: String,
        pub password: String,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum AuthError {
        #[error(transparent)]
        Sqlx(#[from] sqlx::Error),

        #[error(transparent)]
        TaskJoin(#[from] task::JoinError),

        #[error(transparent)]
        Bcrypt(#[from] bcrypt::BcryptError),
    }

    #[async_trait]
    impl AuthnBackend for Backend{
        type User = SqlUser;
        type Credentials = Credentials;
        type Error = AuthError;

        async fn authenticate(
            &self,
            creds: Self::Credentials,
        ) -> Result<Option<Self::User>, Self::Error> {
            let sql_user: Option<SqlUser> = sqlx::query_as::<_, SqlUser>(
                "SELECT * FROM users WHERE username = ?",
            )
                .bind(creds.username)
                .fetch_optional(&self.pool)
                .await?;

            // Verifying the password is blocking and potentially slow, so we'll do so via
            // `spawn_blocking`.
            task::spawn_blocking(|| {
                let user = sql_user.unwrap();
                if verify(creds.password, &user.password)?{
                    return Ok(Some(user))
                }else{
                    return Ok(None)
                }
            })
            .await?
        }

        async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
            let sql_user = sqlx::query_as::<_, SqlUser>(
                "SELECT * FROM users WHERE id = ?",
            )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

            Ok(sql_user)
        }
    }

    #[derive(sqlx::FromRow, Clone)]
    pub struct SqlUser {
        pub id: i64,
        pub username: String,
        pub email: String,
        pub password: String,
        pub signed_up: bool,
        pub admin: bool,
    }

    // Here we've implemented `Debug` manually to avoid accidentally logging the
    // password hash.
    impl std::fmt::Debug for SqlUser {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("User")
                .field("id", &self.id)
                .field("username", &self.username)
                .field("password", &"[redacted]")
                .finish()
        }
    }

    impl SqlUser {
        pub async fn get(id: i64, pool: &SqlitePool) -> Option<Self> {
            let sqluser = sqlx::query_as::<_, SqlUser>(
                "SELECT * FROM users WHERE id = ?",
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .ok()?;

            Some(sqluser)
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

            Some(sqluser)
        }

        pub fn into_user(
            self,
        ) -> User {
            User {
                id: self.id,
                username: self.username,
                email: self.email,
                permissions: if self.admin == true {
                    ["admin".to_string()]
                    .into_iter()
                    .collect::<HashSet<String>>()
                } else {
                    HashSet::<String>::new()
                }
            }
        }
    }
}

#[server]
pub async fn get_user() -> Result<Option<User>, ServerFnError> {
    use ssr::auth;
    let auth = auth()?;
    Ok(auth.user.map_or(None, |u| Some(u.into_user())))
}
