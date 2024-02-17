use duckdb::Connection;

static DB: std::sync::RwLock<Connection> = std::sync::RwLock::<Connection>::new();

pub async fn init_db() -> () {
    let conn = Connection::open("database.db").expect("Could not initialize database");
    //DB = std::sync::RwLock::new(conn);
    let db: std::sync::RwLock<Connection> = std::sync::RwLock::<Connection>::new(conn);
}

pub fn get_db() -> &Connection {
    DB.get().expect("Database uninitialized")
}
