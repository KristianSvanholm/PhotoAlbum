CREATE TABLE IF NOT EXISTS users (
  id         INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  username   TEXT NOT NULL UNIQUE,
  email      TEXT NOT NULL UNIQUE,
  password   TEXT NOT NULL,
  profilePic BLOB NULL,
  admin      BOOLEAN NOT NULL DEFAULT 0,
  internal   BOOLEAN NOT NULL DEFAULT 0,
  invited    BOOLEAN NOT NULL DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS user_permissions (
    user_id  INTEGER NOT NULL REFERENCES users(id),
    token    TEXT NOT NULL
);

create table IF NOT EXISTS folder(
    id uuid primary key not null,
    parentId uuid references folder(id) null,
    name text not null,
    createdDate timestamp not null
);

create table IF NOT EXISTS file(
    id uuid primary key not null,
    folderId integer references folder(id) not null,
    path text not null,
    location POINT_2D null,
    uploadedBy INTEGER references user(id) null,
    uploadDate timestamp not null,
    createdDate timestamp not null
);

create table IF NOT EXISTS userFile(
    userID INTEGER references user(id) not null,
    fileID uuid references file(id) not null,
    primary key(userID, fileID)
);

create table IF NOT EXISTS tag (
    tagString text primary key not null
);

create table IF NOT EXISTS tagFile (
    tagString text references tag(tagString) not null,
    fileID uuid references file(id) not null,
    primary key(tagString, fileID)
);
