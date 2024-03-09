CREATE TABLE IF NOT EXISTS invites (
    token TEXT NOT NULL UNIQUE PRIMARY KEY,
    userID INTEGER NOT NULL REFERENCES users(id),
    admin BOOLEAN NOT NULL DEFAULT 0
);