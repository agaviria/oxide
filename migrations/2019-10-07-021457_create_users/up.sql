-- Define the create users table
DROP EXTENSION
IF EXISTS pgcrypto;
CREATE EXTENSION pgcrypto;

CREATE TABLE users
(
    uuid UUID PRIMARY KEY NOT NULL Default gen_random_uuid(),
    user_name VARCHAR NOT NULL UNIQUE,
    display_name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    password VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT Now(),
    updated_at TIMESTAMP NOT NULL DEFAULT Now(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE
);

-- Indices

CREATE INDEX idx_users_id ON users (uuid);
CREATE INDEX idx_users_email ON users (email);