-- Drop existing table if it exists
DROP TABLE IF EXISTS todos;
DROP TABLE IF EXISTS users;

-- Create users table
CREATE TABLE users
(
    id         SERIAL PRIMARY KEY,
    uuid       VARCHAR(255) UNIQUE      NOT NULL,
    email      VARCHAR(255)             NOT NULL UNIQUE,
    name       VARCHAR(255)             NOT NULL,
    password   VARCHAR(255)             NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
); 