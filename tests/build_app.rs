
use axum::{
    body::Body as AxumBody,
    extract::{Path, State},
    http::Request,
    response::{IntoResponse, Response},
    routing::get,
    Router,
    middleware,
};
use axum_login::{
    AuthManagerLayerBuilder,
    tower_sessions::{Expiry, SessionManagerLayer, Session}
};
use tower_sessions_sqlx_store::SqliteStore;
use leptos::{get_configuration, logging::log, provide_context};
use leptos_axum::{
    generate_route_list, handle_server_fns_with_context, LeptosRoutes,
};
use photo_album::{
    auth::ssr::{AuthSession, Backend},
    session::session_expiry::SessionExpiryConfig,
    state::AppState,
    app::*,
};
use sqlx::sqlite::SqlitePoolOptions;

async fn server_fn_handler(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    session: Session,
    path: Path<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    log!("{:?}", path);

    handle_server_fns_with_context(
        move || {
            provide_context(auth_session.clone());
            provide_context(session.clone());
            provide_context(app_state.expiry_config.clone());
            provide_context(app_state.pool.clone());
        },
        request,
    )
    .await
}

async fn leptos_routes_handler(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    req: Request<AxumBody>,
) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pool.clone());
        },
        App,
    );
    handler(req).await.into_response()
}

pub async fn build_app() -> Router{
    use photo_album::fileserv::file_and_error_handler;
    use photo_album::session::session_expiry::session_expiry_manager;

    let pool = SqlitePoolOptions::new()
        .connect("sqlite:test_database.db")
        .await
        .expect("Could not make pool.");

    let session_store = SqliteStore::new(pool.clone());
    session_store.migrate().await.unwrap();

    // Setting this to None means we'll be using cargo-leptos and its env vars
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let expiry_config: SessionExpiryConfig = Default::default();
    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options,
        expiry_config,
        pool: pool.clone(),
        routes: routes.clone(),
    };

    // build our application with a route
    let app = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .route("/pkg/*path", get(file_and_error_handler))
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .layer(middleware::from_fn_with_state(app_state.clone(), session_expiry_manager))
        .layer(
            AuthManagerLayerBuilder::new(Backend::new(pool), 
                SessionManagerLayer::new(session_store.clone())
                .with_secure(false)
                .with_name("session")
                //needed so the cookie gets a max-age attribute
                .with_expiry(Expiry::OnInactivity(expiry_config.expiry))
            ).build())
        .fallback(file_and_error_handler)
        .with_state(app_state);

    app
}