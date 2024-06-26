CREATE TABLE IF NOT EXISTS invites (
    token TEXT NOT NULL UNIQUE PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    admin_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, 
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE users
DROP COLUMN internal;

ALTER TABLE users 
RENAME COLUMN invited TO signed_up;