-- SQL goes here

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email  VARCHAR UNIQUE NOT NULL,
    hashed_pwd VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT Now(),
    CHECK(length(password) >= 6)
);
