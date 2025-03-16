use crate::db::data_trait::user_data_trait::UserData;
use crate::db::database::Database;
use crate::error::user_error::UserError;
use crate::models::user::{
    CreateUserRequest, LoginRequest, UpdateUserRequest, UpdateUserURL, User, UserResponse,
};
use actix_web::{
    patch, post,
    web::{Data, Json, Path},
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub sub: String,
    #[schema(example = "1633027200")]
    pub exp: usize,
}

pub fn user_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(create_user);
    cfg.service(login_user);
    cfg.service(update_user);
}

/// Đăng ký người dùng mới
///
/// Endpoint này cho phép đăng ký một người dùng mới với tên người dùng và mật khẩu.
#[utoipa::path(
    post,
    path = "/api/v1/signup",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "Người dùng được tạo thành công", body = UserResponse),
        (status = 400, description = "Dữ liệu không hợp lệ"),
        (status = 500, description = "Lỗi máy chủ")
    )
)]
#[post("/signup")]
async fn create_user(
    body: Json<CreateUserRequest>,
    db: Data<Database>,
) -> Result<Json<UserResponse>, UserError> {
    let is_valid = body.validate();
    match is_valid {
        Ok(_) => {
            let username = body.username.clone();
            let mut buffer = Uuid::encode_buffer();
            let new_uuid = Uuid::new_v4()
                .simple()
                .encode_lower(&mut buffer)
                .to_string();
            // hash password
            let hashed_password = match hash(&body.password, DEFAULT_COST) {
                Ok(hash) => hash,
                Err(e) => {
                    eprintln!("Password hashing failed: {:?}", e);
                    return Err(UserError::UserCreationFailure);
                }
            };
            let expiration = Utc::now()
                .checked_add_signed(Duration::hours(24))
                .expect("Valid timestamp")
                .timestamp() as usize;

            let claims = Claims {
                sub: new_uuid.clone(),
                exp: expiration,
            };

            let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_key".into());
            let token = match encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret.as_ref()),
            ) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("JWT token creation failed: {:?}", e);
                    return Err(UserError::UserCreationFailure);
                }
            };

            let new_user = Database::add_user(
                &db,
                User::new(
                    new_uuid,
                    username,
                    hashed_password,
                    Utc::now().to_string(),
                    Utc::now().to_string(),
                ),
            )
            .await;

            match new_user {
                Some(user) => Ok(Json(UserResponse {
                    username: user.username,
                    access_token: token,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                })),
                None => Err(UserError::UserCreationFailure),
            }
        }
        Err(_) => Err(UserError::UserCreationFailure),
    }
}

/// Đăng nhập người dùng
///
/// Endpoint này cho phép người dùng đăng nhập với tên người dùng và mật khẩu.
#[utoipa::path(
    post,
    path = "/api/v1/login",
    tag = "users",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Đăng nhập thành công", body = UserResponse),
        (status = 400, description = "Dữ liệu không hợp lệ"),
        (status = 401, description = "Xác thực thất bại"),
        (status = 404, description = "Không tìm thấy người dùng"),
        (status = 500, description = "Lỗi máy chủ")
    )
)]
#[post("/login")]
async fn login_user(
    body: Json<LoginRequest>,
    db: Data<Database>,
) -> Result<Json<UserResponse>, UserError> {
    // Find user by username
    let query = "SELECT uuid, username, password, created_at::TEXT as created_at, updated_at::TEXT as updated_at FROM users WHERE username = $1";

    let user_result = sqlx::query(query)
        .bind(&body.username)
        .fetch_optional(&db.pool)
        .await;

    match user_result {
        Ok(Some(row)) => {
            let user = User {
                uuid: row.get("uuid"),
                username: row.get("username"),
                password: row.get("password"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            // Verify password
            match verify(&body.password, &user.password) {
                Ok(true) => {
                    // Generate JWT token
                    let expiration = Utc::now()
                        .checked_add_signed(Duration::hours(24))
                        .expect("Valid timestamp")
                        .timestamp() as usize;

                    let claims = Claims {
                        sub: user.uuid.clone(),
                        exp: expiration,
                    };

                    let secret =
                        std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_key".into());
                    let token = match encode(
                        &Header::default(),
                        &claims,
                        &EncodingKey::from_secret(secret.as_ref()),
                    ) {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("JWT token creation failed: {:?}", e);
                            return Err(UserError::AuthenticationFailure);
                        }
                    };

                    Ok(Json(UserResponse {
                        username: user.username,
                        access_token: token,
                        created_at: user.created_at,
                        updated_at: user.updated_at,
                    }))
                }
                Ok(false) => Err(UserError::AuthenticationFailure),
                Err(_) => Err(UserError::AuthenticationFailure),
            }
        }
        Ok(None) => Err(UserError::NoSuchUserFound),
        Err(e) => {
            eprintln!("Error finding user: {:?}", e);
            Err(UserError::AuthenticationFailure)
        }
    }
}

/// Cập nhật thông tin người dùng
///
/// Endpoint này cho phép cập nhật thông tin của người dùng.
#[utoipa::path(
    patch,
    path = "/api/v1/users/{uuid}",
    tag = "users",
    params(
        ("uuid" = String, Path, description = "UUID của người dùng cần cập nhật")
    ),
    request_body = UpdateUserRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Người dùng được cập nhật thành công", body = User),
        (status = 400, description = "Dữ liệu không hợp lệ"),
        (status = 401, description = "Không được xác thực"),
        (status = 404, description = "Không tìm thấy người dùng"),
        (status = 500, description = "Lỗi máy chủ")
    )
)]
#[patch("/users/{uuid}")]
async fn update_user(
    update_user_url: Path<UpdateUserURL>,
    body: Json<UpdateUserRequest>,
    db: Data<Database>,
) -> Result<Json<User>, UserError> {
    let is_valid = body.validate();
    match is_valid {
        Ok(_) => {
            let updated_user =
                Database::update_user(&db, update_user_url.uuid.clone(), body.username.clone())
                    .await;

            match updated_user {
                Some(user) => Ok(Json(user)),
                None => Err(UserError::NoSuchUserFound),
            }
        }
        Err(_) => Err(UserError::NoSuchUserFound),
    }
}
