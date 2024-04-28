CREATE TABLE IF NOT EXISTS users (
  id         INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  username   TEXT NOT NULL UNIQUE,
  email      TEXT NULL,
  password   TEXT NULL,
  profilePic BLOB NULL,
  admin      BOOLEAN NOT NULL DEFAULT 0,
  internal   BOOLEAN NOT NULL DEFAULT 0,
  invited    BOOLEAN NOT NULL DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS user_permissions (
    user_id  INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token    TEXT NOT NULL
);

create table IF NOT EXISTS folders(
    id uuid primary key not null,
    parentId uuid references folder(id) null,
    name text not null,
    createdDate timestamp not null
);

create table IF NOT EXISTS files(
    id uuid primary key not null,
    folderId integer references folders(id) null,
    path text not null,
    location POINT_2D null,
    uploadedBy INTEGER references users(id) null,
    uploadDate timestamp not null,
    createdDate timestamp not null
);

create table IF NOT EXISTS userFile(
    userID INTEGER references users(id) not null,
    fileID uuid references files(id) not null,
    primary key(userID, fileID)
);

create table IF NOT EXISTS tags (
    tagString text primary key not null
);

create table IF NOT EXISTS tagFile (
    tagString text references tags(tagString) not null,
    fileID uuid references files(id) not null,
    primary key(tagString, fileID)
);
