-- DROP TABLE IF EXISTS todos;
-- DROP TABLE IF EXISTS users;
-- Create users table if not exists
CREATE TABLE IF NOT EXISTS users
(
    id         SERIAL PRIMARY KEY,
    uuid       VARCHAR(255) UNIQUE      NOT NULL,
    email      VARCHAR(255)             NOT NULL UNIQUE,
    name       VARCHAR(255)             NOT NULL,
    password   VARCHAR(255)             NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Create todos table if not exists
CREATE TABLE IF NOT EXISTS todos
(
    id           SERIAL PRIMARY KEY,
    uuid         VARCHAR(255) UNIQUE      NOT NULL,
    title        VARCHAR(255)             NOT NULL,
    description  TEXT                     NOT NULL,
    is_completed BOOLEAN                  NOT NULL DEFAULT FALSE,
    owner_id     VARCHAR(255)             NOT NULL,
    created_at   TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at   TIMESTAMP WITH TIME ZONE NOT NULL,
    CONSTRAINT fk_owner FOREIGN KEY (owner_id) REFERENCES users (uuid)
); 