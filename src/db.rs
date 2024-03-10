#[cfg(feature = "ssr")]
pub mod ssr {
    pub use sqlx::SqlitePool;
    pub use leptos::*;

    pub fn pool() -> Result<SqlitePool, ServerFnError> {
        use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
    }
}
