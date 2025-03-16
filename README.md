# Rust Backend API

A RESTful API built with Rust and Actix Web, providing user management and todo list functionality.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Technologies](#technologies)
- [Installation](#installation)
- [Configuration](#configuration)
- [Project Structure](#project-structure)
- [API Endpoints](#api-endpoints)
- [Authentication](#authentication)
- [API Documentation](#api-documentation)
- [Development](#development)
- [Deployment](#deployment)
- [Contributing](#contributing)
- [License](#license)

## Overview

Rust Backend API is a backend application built with Rust, using the Actix Web framework and PostgreSQL database. The
application provides APIs for user management and todo lists with complete CRUD (Create, Read, Update, Delete)
functionality.

## Features

- **User Management**:
    - Register new accounts
    - Login with JWT authentication
    - Update user information

- **Todo Management**:
    - Create new todos
    - List todos with pagination and filtering
    - View todo details
    - Update todos
    - Delete todos

- **Additional Features**:
    - JWT authentication
    - API documentation with Swagger UI
    - System health check

## Technologies

- **Language**: [Rust](https://www.rust-lang.org/)
- **Web Framework**: [Actix Web](https://actix.rs/)
- **Database**: [PostgreSQL](https://www.postgresql.org/)
- **ORM**: [SQLx](https://github.com/launchbadge/sqlx)
- **Authentication**: [jsonwebtoken](https://github.com/Keats/jsonwebtoken)
- **Password Hashing**: [bcrypt](https://github.com/Keats/rust-bcrypt)
- **API Documentation**: [utoipa](https://github.com/juhaku/utoipa)
  and [Swagger UI](https://swagger.io/tools/swagger-ui/)
- **Logging**: [env_logger](https://github.com/env-logger-rs/env_logger/)
- **Error Handling**: Custom error handling
- **Validation**: [validator](https://github.com/Keats/validator)

## Installation

### Requirements

- Rust (version 1.60.0 or higher)
- Cargo (comes with Rust)
- PostgreSQL (version 12 or higher)

### Setup Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/nqmgaming/todo-rust-backend.git
   cd todo-rust-backend
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

3. Set up the database:
    - Create a PostgreSQL database
    - Run the SQL scripts in the `migrations/` directory to create the necessary tables

4. Create a `.env` file from the example:
   ```bash
   cp .env.example .env
   ```

5. Update the configuration in the `.env` file

6. Run the application:
   ```bash
   cargo run
   ```

## Configuration

The application uses environment variables for configuration. You can set the following variables in your `.env` file:

```
# Database
DB_HOST=localhost
DB_PORT=5432
DB_NAME=rust_backend
DB_USER=postgres
DB_PASSWORD=your_password

# JWT
JWT_SECRET=your_jwt_secret_key

# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Logging
RUST_LOG=info
```

## Project Structure

```
rust_backend/
├── src/
│   ├── db/
│   │   ├── data_trait/
│   │   │   ├── todo_data_trait.rs
│   │   │   └── user_data_trait.rs
│   │   └── database.rs
│   ├── error/
│   │   ├── app_error.rs
│   │   └── user_error.rs
│   ├── middleware/
│   │   └── auth.rs
│   ├── models/
│   │   ├── todo.rs
│   │   └── user.rs
│   ├── routers/
│   │   ├── health.rs
│   │   ├── todo.rs
│   │   └── user.rs
│   ├── swagger.rs
│   └── main.rs
├── migrations/
│   ├── 01_create_users_table.sql
│   └── 02_create_todos_table.sql
├── Cargo.toml
├── Cargo.lock
├── .env.example
├── .env
├── Dockerfile
├── docker-compose.yml
└── README.md
```

## API Endpoints

### Health Check

- `GET /api/health` - Check API health status

### User Management

- `POST /api/v1/signup` - Register a new user
- `POST /api/v1/login` - Login
- `PATCH /api/v1/users/{uuid}` - Update user information

### Todo Management

- `GET /api/v1/todos` - Get list of todos
- `GET /api/v1/todos/{uuid}` - Get todo details
- `POST /api/v1/todos` - Create a new todo
- `PATCH /api/v1/todos/{uuid}` - Update todo
- `DELETE /api/v1/todos/{uuid}` - Delete todo

## Authentication

The API uses JWT (JSON Web Token) for authentication. To access protected endpoints:

1. Register a new account or login to get a token
2. Add the token to the request header:
   ```
   Authorization: Bearer your_token_here
   ```

## API Documentation

The API is documented using Swagger UI. After running the application, you can access the API documentation at:

```
http://127.0.0.1:8080/swagger-ui/
```

Here, you can:

- View all available endpoints
- Test APIs directly from the interface
- View schemas and data models
- Authenticate with JWT to test protected endpoints

## Development

### Running Tests

```bash
cargo test
```

### Checking for Errors

```bash
cargo check
```

### Formatting Code

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Deployment

### Docker

The project can be containerized and deployed using Docker:

```bash
# Build and run with docker-compose
docker compose up -d

# Or build and run manually
docker build -t rust-backend .
docker run -p 8080:8080 --env-file .env rust-backend
```

### Cloud Platforms

The project can be deployed to cloud platforms such as:

- AWS (Amazon Web Services)
- Google Cloud Platform
- Microsoft Azure
- Heroku
- DigitalOcean

## Contributing

Contributions are welcome! If you'd like to contribute to the project, please:

1. Fork the repository
2. Create a new branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the `LICENSE` file for details.

---

Developed by [Nguyen Quang Minh (NQM)](https://github.com/nqmgaming)

