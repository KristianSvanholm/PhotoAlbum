use axum::{
    body::Body as AxumBody,
    extract::{Path, State},
    http::Request,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSessionLayer, SessionSqlitePool};
use leptos::{get_configuration, logging::log, provide_context};
use leptos_axum::{
    generate_route_list, handle_server_fns_with_context, LeptosRoutes,
};
use photo_album::{
    auth::{ssr::AuthSession, User},
    state::AppState,
    app::*,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn server_fn_handler(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    path: Path<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    log!("{:?}", path);

    handle_server_fns_with_context(
        move || {
            provide_context(auth_session.clone());
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
async fn add_first_user(
    username: String,
    pool: &SqlitePool
){
    #[derive(sqlx::FromRow)]
    struct Res{
        IsEmpty: bool 
    }

    let users_is_empty = sqlx::query_as::<_,Res>("SELECT CASE WHEN EXISTS(SELECT 1 FROM users) THEN 0 ELSE 1 END AS IsEmpty;")
        .fetch_one(pool)
        .await
        .expect("Database call failed");

    println!("Is empty = {}",users_is_empty.IsEmpty);

    if users_is_empty.IsEmpty{
        sqlx::query("INSERT INTO users (username, admin) VALUES (?, 1)")
            .bind(&username)
            .execute(pool)
            .await
            .expect("Inserting admin in database failed");

        let id = sqlx::query_scalar("SELECT id FROM users ORDER BY rowid DESC limit 1")
            .fetch_one(pool)
            .await
            .expect("Getting id from database failed");

        println!("Admin ID = {}",id);

        let link = photo_album::components::invite::create_invitation_link(&id, &id, &pool)
            .await
            .expect("Getting invite_link failed");

        println!("Admin with username {name} was added", name = username);
        println!("Sign_up now using the following link: {link}", link = link);
    }else{
        println!("Database is not empty, no addional admins are inserted.");
    }
}

#[tokio::main]
async fn main() {
    use photo_album::fileserv::file_and_error_handler;
    use std::fs::File;
    use std::path::Path;

    simple_logger::init_with_level(log::Level::Info)
        .expect("couldn't initialize logging");
    
    if !Path::new("database.db").exists() {
        let _ = File::create("database.db");
    }

    let pool = SqlitePoolOptions::new()
        .connect("sqlite:database.db")
        .await
        .expect("Could not make pool.");

    // Auth section
    let session_config =
        SessionConfig::default().with_table_name("axum_sessions");
    let auth_config = AuthConfig::<i64>::default();
    let session_store = SessionStore::<SessionSqlitePool>::new(
        Some(SessionSqlitePool::from(pool.clone())),
        session_config,
    )
    .await
    .unwrap();
 
    if let Err(e) = sqlx::migrate!().run(&pool).await {
        eprintln!("{e:?}");
    }
    
    //initalize first admin onfirst run
    add_first_user("admin".to_string(), &pool).await;

    // Setting this to None means we'll be using cargo-leptos and its env vars
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options,
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
        .layer(
            AuthSessionLayer::<User, i64, SessionSqlitePool, SqlitePool>::new(
                Some(pool.clone()),
            )
            .with_config(auth_config),
        )
        .fallback(file_and_error_handler)
        .layer(SessionLayer::new(session_store))
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
