use axum::{
    body::Body as AxumBody,
    extract::{Path, State},
    http::Request,
    middleware,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_login::{
    tower_sessions::{ExpiredDeletion, Expiry, Session, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use leptos::{get_configuration, logging::log, provide_context};
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
use photo_album::{
    app::*,
    auth::ssr::{AuthSession, Backend},
    session::session_expiry::SessionExpiryConfig,
    state::AppState,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tokio::{signal, task::AbortHandle};
use tower_sessions_sqlx_store::SqliteStore;

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

#[cfg(feature = "ssr")]
async fn add_first_user(username: String, pool: &SqlitePool) {
    let users_is_empty: bool = sqlx::query_scalar(
        "SELECT CASE WHEN EXISTS(SELECT 1 FROM users) THEN 0 ELSE 1 END AS IsEmpty;",
    )
    .fetch_one(pool)
    .await
    .expect("Database call failed");

    if !users_is_empty {
        println!("Database is not empty, no addional admins are inserted.");
        return;
    }

    sqlx::query("INSERT INTO users (username, admin) VALUES (?, 1)")
        .bind(&username)
        .execute(pool)
        .await
        .expect("Inserting admin in database failed");

    let id = sqlx::query_scalar("SELECT id FROM users ORDER BY rowid DESC limit 1")
        .fetch_one(pool)
        .await
        .expect("Getting id from database failed");

    let link = photo_album::components::invite::create_invitation_link(&id, &id, &pool)
        .await
        .expect("Getting invite_link failed");

    println!("Admin with username {name} was added", name = username);
    println!("Sign_up now using the following link: http://0.0.0.0:3000{link}", link = link);
}

#[tokio::main]
async fn main() {
    use photo_album::fileserv::file_and_error_handler;
    use photo_album::session::session_expiry::session_expiry_manager;
    use std::fs::File;
    use std::path::Path;

    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

    let db_path = "/app/data/database.db";
    if !Path::new(db_path).exists() {
        let _ = File::create(db_path);
    }

    let pool = SqlitePoolOptions::new()
        .connect(format!("sqlite://{}", db_path).as_str())
        .await
        .expect("Could not make pool.");

    // Auth section
    let session_store = SqliteStore::new(pool.clone());
    session_store.migrate().await.unwrap();
    let deletion_task = tokio::task::spawn(ExpiredDeletion::continuously_delete_expired(
        session_store.clone(),
        tokio::time::Duration::from_secs(12 * 60 * 60),
    ));

    if let Err(e) = sqlx::migrate!().run(&pool).await {
        eprintln!("{e:?}");
    }

    //initalize first admin onfirst run
    add_first_user("admin".to_string(), &pool).await;

    // Setting this to None means we'll be using cargo-leptos and its env vars
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let expiry_config: SessionExpiryConfig = Default::default();
    let addr = leptos_options.site_addr;
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
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            session_expiry_manager,
        ))
        .layer(
            AuthManagerLayerBuilder::new(
                Backend::new(pool),
                SessionManagerLayer::new(session_store.clone())
                    .with_secure(false)
                    .with_name("session")
                    //needed so the cookie gets a max-age attribute
                    .with_expiry(Expiry::OnInactivity(expiry_config.expiry)),
            )
            .build(),
        )
        .fallback(file_and_error_handler)
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    // Ensure we use a shutdown signal to bort the deletion task.
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
        .await
        .unwrap();

    deletion_task.await.unwrap().unwrap();
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}
