INSTALL spatial;
LOAD spatial;

create sequence seq_userid start 1;

create table user(
    id integer primary key default nextval('seq_userid') not null,
    email varchar(255) not null,
    realName varchar(100) not null,
    password varchar(255) not null,
    hash varchar(255) not null,
    profilePic blob null,
    admin boolean not null default 0,
    internal boolean not null default 0,
    invited boolean not null default 0
);

create sequence seq_folderid start 1;

create table folder(
    id integer primary key default nextval('seq_folderid') not null,
    parentId integer references folder(id) null,
    name varchar(75) not null,
    createdDate timestamp default current_timestamp not null
);

create sequence seq_fileid start 1;

create table file(
    id integer primary key default nextval('seq_fileid') not null,
    folderId integer references folder(id) not null,
    path varchar(500) not null,
    location POINT_2D null,
    uploadedBy integer references user(id) null,
    uploadDate timestamp not null,
    createdDate timestamp not null
);

create table userFile(
    userID integer references user(id) not null,
    fileID integer references file(id) not null,
    primary key(userID, fileID)
);

create table tag (
    tagString varchar(50) primary key not null
);

create table tagFile (
    tagString varchar(50) references tag(tagString) not null,
    fileID integer references file(id) not null,
    primary key(tagString, fileID)
);