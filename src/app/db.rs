#[cfg(feature = "ssr")]
use rusqlite::Connection;

#[cfg(feature = "ssr")]
pub async fn db() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open("database.db");
    conn
}

// Takes a create table query and prints out its specified name.
#[cfg(feature = "ssr")]
fn log_table_name(query: &str){
    use leptos::logging;
    logging::log!("{}", query
        .split("table").collect::<Vec<_>>()[1] // Boldly assumes there is a second part :)
        .split("(").collect::<Vec<_>>()[0]);
}

// Initializes DB with tables if not already present.
#[cfg(feature = "ssr")]
pub async fn db_init() -> Result<(), rusqlite::Error> {
    use leptos::*;
    use std::fs;
    use std::path::Path;

    // If database already exists, return early.
    if Path::new("database.db").exists() {
        logging::log!("DB already exists");
        return Ok(());
    }

    // Get table strings
    let contents = fs::read_to_string("db.sql")
        .expect("Missing db.sql file in root.");
    let tables = contents.split(";");

    // Connect to DB
    let conn = db().await.expect("Could not create db file");

    // Create tables
    for table in tables {
        match conn.execute(table, ()) {
            Ok(_) => log_table_name(table),
            Err(e) => return Err(e),
        };
    }

    Ok(())
}
