use bcrypt::{hash, DEFAULT_COST};
use sqlx::sqlite::SqlitePoolOptions;
use std::fs::{remove_file, File};

#[cfg(test)]
pub async fn prepare_database() {
    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

    remove_file("test_database.db").unwrap();
    let _ = File::create_new("test_database.db");

    let pool = SqlitePoolOptions::new()
        .connect("test_database.db")
        .await
        .expect("Could not make pool.");

    if let Err(e) = sqlx::migrate!().run(&pool).await {
        eprintln!("{e:?}");
    }

    //add admin and user
    let password_hashed = hash("admin", DEFAULT_COST).unwrap();
    sqlx::query(
        "INSERT INTO users (id, username, email, password, admin, signed_up) 
        VALUES (1, 'admin', 'admin@test', ?, 1, 1)",
    )
    .bind(password_hashed)
    .execute(&pool)
    .await
    .expect("Inserting admin in database failed");

    let password_hashed = hash("user", DEFAULT_COST).unwrap();
    sqlx::query(
        "INSERT INTO users (id, username, email, password, admin, signed_up) 
        VALUES (2, 'user', 'user@test', ?, 0, 1)",
    )
    .bind(password_hashed)
    .execute(&pool)
    .await
    .expect("Inserting user in database failed");
}
