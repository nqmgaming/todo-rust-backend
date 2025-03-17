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

# Rust Backend với Xác thực Hai Yếu Tố (2FA)

Dự án này là một backend API được xây dựng bằng Rust với tính năng xác thực hai yếu tố (2FA) sử dụng TOTP (Time-based One-Time Password).

## Tính năng

- Đăng ký và đăng nhập người dùng
- Xác thực JWT
- Xác thực hai yếu tố (2FA) với TOTP
- Quản lý phiên làm việc với Redis
- Cơ sở dữ liệu PostgreSQL

## Cài đặt

### Yêu cầu

- Rust và Cargo
- PostgreSQL
- Redis

### Cài đặt

1. Clone repository:
```bash
git clone https://github.com/yourusername/rust_backend.git
cd rust_backend
```

2. Cấu hình biến môi trường:
```bash
cp .env.example .env
```

3. Chỉnh sửa file `.env` với thông tin cấu hình của bạn.

4. Chạy migration cơ sở dữ liệu:
```bash
psql -U your_username -d your_database -f migrations/add_2fa_to_users.sql
```

5. Xây dựng và chạy ứng dụng:
```bash
cargo run
```

## API Endpoints

### Đăng ký người dùng
```
POST /api/v1/register
```

### Đăng nhập
```
POST /api/v1/login
```

### Làm mới token
```
POST /api/v1/refresh
```

### Bật 2FA
```
POST /api/v1/users/{uuid}/enable-2fa
```

### Xác minh 2FA
```
POST /api/v1/users/{uuid}/verify-2fa
```

### Tắt 2FA
```
POST /api/v1/users/{uuid}/disable-2fa
```

## Luồng xác thực 2FA

1. **Bật 2FA**:
   - Người dùng gọi endpoint `enable-2fa` với mật khẩu của họ
   - Hệ thống tạo secret key và QR code
   - Người dùng quét QR code bằng ứng dụng Google Authenticator

2. **Xác minh 2FA**:
   - Người dùng nhập mã từ Google Authenticator vào endpoint `verify-2fa`
   - Hệ thống xác minh mã và bật 2FA cho tài khoản

3. **Đăng nhập với 2FA**:
   - Người dùng đăng nhập với email và mật khẩu
   - Nếu 2FA được bật, hệ thống yêu cầu mã TOTP
   - Người dùng cung cấp mã TOTP từ Google Authenticator
   - Hệ thống xác minh mã và cấp token đăng nhập

4. **Tắt 2FA**:
   - Người dùng gọi endpoint `disable-2fa` với mật khẩu và mã TOTP
   - Hệ thống xác minh thông tin và tắt 2FA cho tài khoản

## Triển khai

Tính năng 2FA được triển khai bằng cách sử dụng các thư viện sau:
- `totp-rs`: Tạo và xác minh mã TOTP
- `qrcode`: Tạo QR code
- `image`: Xử lý hình ảnh
- `base64`: Mã hóa hình ảnh QR code
- `data-encoding`: Mã hóa Base32 cho secret key

## Bảo mật

- Secret key được lưu trữ trong cơ sở dữ liệu
- Mật khẩu được băm bằng bcrypt
- Token JWT được sử dụng cho xác thực
- Redis được sử dụng để quản lý trạng thái token

## Đóng góp

Đóng góp và báo cáo lỗi được chào đón. Vui lòng mở issue hoặc pull request.

