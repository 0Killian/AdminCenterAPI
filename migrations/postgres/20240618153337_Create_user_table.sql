-- Add migration script here
-- POSTGRESQL

-- users(id[PK,NN], username[UQ,NN], password[NN])
CREATE TABLE IF NOT EXISTS users
(
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(32) UNIQUE NOT NULL,
    password VARCHAR(128) NOT NULL
);

-- Default user
INSERT INTO users (id, username, password)
VALUES (1, 'admin', '$argon2id$v=19$m=16,t=2,p=1$S1k0SWF3a3p6WkdnUnFSYw$QSye3SQBbIFlywv3rXX4yQ')