use crate::db::data_trait::user_data_trait::UserData;
use crate::db::database::Database;
use crate::db::redis_client::RedisClient;
use crate::error::user_error::UserError;
use crate::models::user::{
    CreateUserRequest, LoginRequest, RefreshTokenRequest, TokenResponse, UpdateUserRequest,
    UpdateUserURL, User, UserResponse,
};
use actix_web::{
    patch, post,
    web::{Data, Json, Path},
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub sub: String,
    #[schema(example = "1633027200")]
    pub exp: usize,
    #[schema(example = "access")]
    pub token_type: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: Option<String>,
}

// Hàm tiện ích để tạo JWT token
fn generate_jwt_token(
    subject: &str,
    token_type: &str,
    expires_in_hours: i64,
    user_id: Option<&str>,
) -> Result<String, UserError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expires_in_hours))
        .expect("Valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: subject.to_string(),
        exp: expiration,
        token_type: token_type.to_string(),
        user_id: user_id.map(|id| id.to_string()),
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_key".into());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| {
        eprintln!("JWT token creation failed: {:?}", e);
        UserError::AuthenticationFailure
    })
}

// Hàm tiện ích để tạo cặp access token và refresh token
async fn generate_token_pair(
    user_id: &str,
    redis_client: &RedisClient,
) -> Result<(String, String), UserError> {
    // Tạo token_id duy nhất cho refresh token
    let token_id = Uuid::new_v4().to_string();

    // Tạo access token như trước
    let access_token = generate_jwt_token(user_id, "access", 1, None)?;

    // Tạo refresh token với token_id trong payload và user_id trong claims
    let refresh_token = generate_jwt_token(&token_id, "refresh", 24 * 7, Some(user_id))?;

    // Lưu trạng thái token vào Redis
    redis_client
        .store_token_state(&token_id, user_id, 7 * 24 * 60 * 60) // 7 ngày
        .await
        .map_err(|e| {
            eprintln!("Redis error: {:?}", e);
            UserError::TokenCreationFailure
        })?;

    Ok((access_token, refresh_token))
}

// Hàm tiện ích để xác thực refresh token
async fn validate_refresh_token(
    token: &str,
    redis_client: &RedisClient,
) -> Result<String, UserError> {
    // Giải mã JWT để lấy token_id và user_id
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_key".into());

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| UserError::InvalidRefreshToken)?;

    // Kiểm tra loại token
    if token_data.claims.token_type != "refresh" {
        return Err(UserError::InvalidRefreshToken);
    }

    let token_id = token_data.claims.sub;
    let user_id = token_data
        .claims
        .user_id
        .ok_or(UserError::InvalidRefreshToken)?;

    // Kiểm tra và vô hiệu hóa token trong Redis
    match redis_client.validate_and_invalidate_token(&token_id).await {
        Ok(Some(stored_user_id)) => {
            // Kiểm tra xem user_id trong token có khớp với user_id trong Redis không
            if stored_user_id != user_id {
                return Err(UserError::InvalidRefreshToken);
            }
            Ok(user_id)
        }
        Ok(None) => {
            // Token không tồn tại hoặc đã bị vô hiệu hóa
            Err(UserError::InvalidRefreshToken)
        }
        Err(e) => {
            eprintln!("Redis error: {:?}", e);
            Err(UserError::AuthenticationFailure)
        }
    }
}

pub fn user_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(refresh_token_endpoint);
}

/// Đăng ký người dùng mới
///
/// Endpoint này cho phép đăng ký một người dùng mới với tên người dùng và mật khẩu.
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Invalid data"),
        (status = 409, description = "User already exists"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "User"
)]
#[post("/register")]
pub async fn register(
    body: Json<CreateUserRequest>,
    db: Data<Database>,
) -> Result<Json<UserResponse>, UserError> {
    // Validate request
    body.validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    // Check if user already exists
    let existing_user = db.get_user_by_email(&body.email).await?;
    if existing_user.is_some() {
        return Err(UserError::UserAlreadyExists);
    }

    // Hash password
    let hashed_password = hash(&body.password, DEFAULT_COST).map_err(|e| {
        eprintln!("Password hashing error: {:?}", e);
        UserError::PasswordHashingFailure
    })?;

    // Create new user
    let new_uuid = Uuid::new_v4().to_string();
    let user = CreateUserRequest {
        email: body.email.clone(),
        password: hashed_password,
        name: body.name.clone(),
    };

    // Save user to database
    db.create_user(&new_uuid, &user).await?;

    // Generate token pair
    let (access_token, refresh_token_str) =
        generate_token_pair(&new_uuid, &db.redis_client).await?;

    let new_user = User::new(
        new_uuid,
        body.email.clone(),
        body.name.clone(),
        Utc::now().naive_utc(),
        Utc::now().naive_utc(),
    );
    let user_response = UserResponse {
        user: new_user.into(),
        access_token: access_token,
        refresh_token: refresh_token_str,
        token_type: "Bearer".to_string(),
    };
    Ok(Json(user_response))
}

/// Đăng nhập người dùng
///
/// Endpoint này cho phép người dùng đăng nhập với tên người dùng và mật khẩu.
#[utoipa::path(
    post,
    path = "/api/v1/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = UserResponse),
        (status = 400, description = "Invalid data"),
        (status = 401, description = "Invalid credentials"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "User"
)]
#[post("/login")]
pub async fn login(
    body: Json<LoginRequest>,
    db: Data<Database>,
) -> Result<Json<UserResponse>, UserError> {
    // Validate request
    body.validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    // Get user by email
    let user_option = db.get_user_by_email(&body.email).await?;
    let user = match user_option {
        Some(user) => user,
        None => return Err(UserError::InvalidCredentials),
    };

    // Verify password
    let password_matches = verify(&body.password, &user.password).map_err(|e| {
        eprintln!("Password verification error: {:?}", e);
        UserError::AuthenticationFailure
    })?;

    if !password_matches {
        return Err(UserError::InvalidCredentials);
    }

    // Generate token pair
    let (access_token, refresh_token_str) =
        generate_token_pair(&user.uuid, &db.redis_client).await?;

    let user_response = UserResponse {
        user: user.into(),
        access_token: access_token,
        refresh_token: refresh_token_str,
        token_type: "Bearer".to_string(),
    };

    Ok(Json(user_response))
}

/// Làm mới token
///
/// Endpoint này cho phép làm mới access token bằng refresh token.
#[utoipa::path(
    post,
    path = "/api/v1/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = TokenResponse),
        (status = 401, description = "Invalid refresh token"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "User"
)]
#[post("/refresh")]
pub async fn refresh_token_endpoint(
    db: Data<Database>,
    body: Json<RefreshTokenRequest>,
) -> Result<Json<TokenResponse>, UserError> {
    // Xác thực refresh token
    let user_id = validate_refresh_token(&body.refresh_token, &db.redis_client).await?;

    // Kiểm tra xem người dùng có tồn tại không
    let user = db.get_user_by_id(&user_id).await?;
    if user.is_none() {
        return Err(UserError::UserNotFound);
    }

    // Tạo cặp token mới
    let (access_token, refresh_token_str) = generate_token_pair(&user_id, &db.redis_client).await?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token: refresh_token_str,
        token_type: "Bearer".to_string(),
    }))
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
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 400, description = "Invalid data"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[patch("/users/{uuid}")]
pub async fn update_user(
    update_user_url: Path<UpdateUserURL>,
    body: Json<UpdateUserRequest>,
    db: Data<Database>,
) -> Result<Json<User>, UserError> {
    // Validate request
    body.validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    // Update user
    let updated_user = db.update_user(&update_user_url.uuid, &body.email).await?;

    match updated_user {
        Some(user) => Ok(Json(user)),
        None => Err(UserError::UserNotFound),
    }
}
