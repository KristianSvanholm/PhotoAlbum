#[cfg(feature = "ssr")]
pub mod ssr {
    pub use leptos::*;
    pub use sqlx::SqlitePool;

    pub fn pool() -> Result<SqlitePool, ServerFnError> {
        use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
    }
}
