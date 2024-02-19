create table user(
    id uuid primary key not null,
    email text not null,
    realName text not null,
    hash text not null,
    salt text not null,
    profilePic blob null,
    admin boolean not null default 0,
    internal boolean not null default 0,
    invited boolean not null default 0
);

create table folder(
    id uuid primary key not null,
    parentId uuid references folder(id) null,
    name text not null,
    createdDate timestamp not null
);

create table file(
    id uuid primary key not null,
    folderId integer references folder(id) not null,
    path text not null,
    location POINT_2D null,
    uploadedBy UUID references user(id) null,
    uploadDate timestamp not null,
    createdDate timestamp not null
);

create table userFile(
    userID uuid references user(id) not null,
    fileID uuid references file(id) not null,
    primary key(userID, fileID)
);

create table tag (
    tagString text primary key not null
);

create table tagFile (
    tagString text references tag(tagString) not null,
    fileID uuid references file(id) not null,
    primary key(tagString, fileID)
);
