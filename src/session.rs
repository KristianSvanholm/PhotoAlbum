#[cfg(feature = "ssr")]
pub mod session_expiry {
    use crate::state::AppState;
    use axum::{
        extract::{Request, State},
        middleware::Next,
        response::Response,
    };
    pub use axum_login::tower_sessions::{Expiry, Session, SessionManagerLayer};
    use leptos::{use_context, ServerFnError};
    use time::{Duration, OffsetDateTime};

    const EXPIRY_KEY: &str = "expiry";
    const LAST_UPDATED_KEY: &str = "last_updated";

    pub async fn session_expiry_manager(
        State(state): State<AppState>,
        session: Session,
        request: Request,
        next: Next,
    ) -> Response {
        // handle `request`

        // call next layer
        let response = next.run(request).await;

        // handle `response`
        if !session.is_empty().await {
            let _ = update_session_expiry(&session, &state.expiry_config).await;
            // correct sessions expiry date with stored expiry, if the session was modified
            // (if the session is not modified if won't be saved anyway)
            if session.is_modified() {
                let expiry: Expiry = session
                    .get(EXPIRY_KEY)
                    .await
                    .unwrap()
                    .unwrap_or_else(|| Expiry::OnInactivity(state.expiry_config.expiry));
                session.set_expiry(Some(expiry));
            }
        }

        // return response
        response
    }

    pub fn session() -> Result<Session, ServerFnError> {
        use_context::<Session>()
            .ok_or_else(|| ServerFnError::ServerError("Session missing.".into()))
    }

    pub fn expiry_config() -> Result<SessionExpiryConfig, ServerFnError> {
        use_context::<SessionExpiryConfig>()
            .ok_or_else(|| ServerFnError::ServerError("Expiry config missing.".into()))
    }

    pub async fn update_session() -> Result<(), ServerFnError> {
        let session = session()?;
        let expiry_config = expiry_config()?;
        update_session_expiry(&session, &expiry_config).await
    }

    async fn update_session_expiry(
        session: &Session,
        expiry_config: &SessionExpiryConfig,
    ) -> Result<(), ServerFnError> {
        let last_updated: OffsetDateTime = session
            .get(LAST_UPDATED_KEY)
            .await
            .unwrap()
            .unwrap_or_else(|| OffsetDateTime::now_utc().saturating_sub(expiry_config.expiry));
        let expiry: Expiry = session
            .get(EXPIRY_KEY)
            .await
            .unwrap()
            .unwrap_or_else(|| Expiry::OnInactivity(expiry_config.expiry));

        if last_updated <= OffsetDateTime::now_utc().saturating_sub(expiry_config.on_activity_check)
        {
            //Update expiry
            session
                .insert(LAST_UPDATED_KEY, OffsetDateTime::now_utc())
                .await
                .unwrap();
            session.cycle_id().await?;
            if let Expiry::OnInactivity(duration) = expiry {
                let expiry = Expiry::OnInactivity(duration);
                session.insert(EXPIRY_KEY, expiry).await.unwrap();
                session.set_expiry(Some(expiry));
            }

            //if long sessions expires change to on Inactivity, so users
            //are not kicked out in the middlde of their work
            if let Expiry::AtDateTime(datetime) = expiry {
                if datetime < OffsetDateTime::now_utc().saturating_add(expiry_config.expiry) {
                    let expiry = Expiry::OnInactivity(expiry_config.expiry);
                    session.insert(EXPIRY_KEY, expiry).await.unwrap();
                    session.set_expiry(Some(expiry));
                }
            }
        }

        Ok(())
    }

    pub async fn make_session_long_term() -> Result<(), ServerFnError> {
        let session = session()?;
        let expiry_config = expiry_config()?;
        let expired_at =
            OffsetDateTime::now_utc().saturating_add(expiry_config.max_age_term_expiry);
        let expiry = Expiry::AtDateTime(expired_at);
        session.insert(EXPIRY_KEY, expiry).await.unwrap();
        session.set_expiry(Some(expiry));
        Ok(())
    }

    #[derive(Debug, Clone, Copy)]
    pub struct SessionExpiryConfig {
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

    impl Default for SessionExpiryConfig {
        fn default() -> Self {
            Self {
                expiry: Duration::hours(3),
                max_age_term_expiry: Duration::days(60),
                on_activity_check: Duration::hours(1),
            }
        }
    }
}
