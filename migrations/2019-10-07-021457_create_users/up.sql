-- Define the create users table

CREATE TABLE users ( 
    id SERIAL PRIMARY KEY NOT NULL,
    username VARCHAR NOT NULL UNIQUE,
    display_name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    password VARCHAR NOT NULL,
    created_at timestamp without time zone not null default (now() at time zone 'utc'),
    updated_at timestamp without time zone not null default (now() at time zone 'utc'),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE
);

-- Indices

CREATE INDEX idx_users_id ON users (id);
CREATE INDEX idx_users_email ON users (email);