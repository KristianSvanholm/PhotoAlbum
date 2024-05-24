
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
    user_id  INTEGER NOT NULL,
    token    TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

create table IF NOT EXISTS folders(
    id uuid primary key not null,
    parentId uuid null,
    name text not null,
    createdDate timestamp not null,
    FOREIGN KEY(parentID) REFERENCES folder(id) ON DELETE CASCADE
);

create table IF NOT EXISTS files(
    id uuid primary key not null,
    folderId integer null,
    path text not null,
    location POINT_2D null,
    uploadedBy INTEGER null,
    uploadDate timestamp not null,
    createdDate timestamp null,
    FOREIGN KEY(folderId) REFERENCES folders(id),
    FOREIGN KEY(uploadedBy) REFERENCES users(id)
);

create table IF NOT EXISTS userFile(
    userID INTEGER not null,
    fileID uuid not null,
    x INTEGER null,
    y INTEGER null,
    width INTEGER null,
    height INTEGER null,
    PRIMARY KEY(userID, fileID),
    FOREIGN KEY(userID) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY(fileID) REFERENCES files(id) ON DELETE CASCADE
);

create table IF NOT EXISTS tags (
    tagString text primary key not null
);

create table IF NOT EXISTS tagFile (
    tagString text not null,
    fileID uuid not null,
    PRIMARY KEY(tagString, fileID),
    FOREIGN KEY(fileID) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY(tagString) REFERENCES tags(tagString) ON DELETE CASCADE
);
