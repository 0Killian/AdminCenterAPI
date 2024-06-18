-- Add migration script here
-- SQLITE

-- users(id[PK,NN], username[UQ,NN], password[NN])
CREATE TABLE IF NOT EXISTS users
(
    id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL unique,
    password TEXT NOT NULL
);

-- Default user
INSERT INTO users (id, username, password)
VALUES (1, 'admin', '$argon2id$v=19$m=16,t=2,p=1$S1k0SWF3a3p6WkdnUnFSYw$QSye3SQBbIFlywv3rXX4yQ')