
#[cfg(feature = "ssr")]
use rusqlite::Connection;

#[cfg(feature = "ssr")]
pub async fn db() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open("database.db");
    conn
}

#[cfg(feature = "ssr")]
pub async fn verify_table() {
    

}

#[cfg(feature = "ssr")]
pub async fn db_init() -> Result<(), rusqlite::Error> {
    use std::path::Path;
    use leptos::*;

    // If database already exists, return early.
    if Path::new("database2.db").exists() {
        logging::log!("DB already exists");
        return Ok(());
    }

    let conn = db().await.expect("Could not create db file");

    // Todo :: Run these create table statements from file. e.g. "db.sql" 
    match conn.execute("
        create table user(
            id integer primary key not null,
            email varchar(255) not null,
            realName varchar(100) not null,
            password varchar(255) not null,
            hash varchar(255) not null,
            profilePic blob null,
            admin boolean not null default 0,
            internal boolean not null default 0,
            invited boolean not null default 0
        );", ()
    ) {
        Ok(_) => logging::log!("Created user"),
        Err(e) => return Err(e)
    };

    match conn.execute("
        create table folder(
            id integer primary key not null,
            parentId uuid references folder(id) null,
            name varchar(75) not null,
            createdDate timestamp default current_timestamp not null
        );", ()
    ) {
        Ok(_) => logging::log!("Created folder"),
        Err(e) => return Err(e)
    };

    match conn.execute("
        create table file(
            id integer primary key not null,
            folderId integer references folder(id) not null,
            path varchar(500) not null,
            location POINT_2D null,
            uploadedBy integer references user(id) null,
            uploadDate timestamp not null,
            createdDate timestamp not null
        );", ()
    ) {
        Ok(_) => logging::log!("Created file"),
        Err(e) => return Err(e)
    };

    match conn.execute("
        create table userFile(
            userID integer references user(id) not null,
            fileID integer references file(id) not null,
            primary key(userID, fileID)
        );", ()
    ) {
        Ok(_) => logging::log!("Create userFile"),
        Err(e) => return Err(e)
    };

    match conn.execute("
        create table tag (
            tagString varchar(50) primary key not null
        );", ()
    ) {
        Ok(_) => logging::log!("Created tag"),
        Err(e) => return Err(e)
    };

    match conn.execute("
        create table tagFile (
            tagString varchar(50) references tag(tagString) not null,
            fileID integer references file(id) not null,
            primary key(tagString, fileID)
        );", ()
    ) {
        Ok(_) => logging::log!("Created tagFile"),
        Err(e) => return Err(e)
    };

    Ok(())
}