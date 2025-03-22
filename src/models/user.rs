use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize, Serialize)]
pub struct CreateUserRequest {
    #[validate(email, length(min = 6, message = "email required"))]
    pub email: String,
    #[validate(length(min = 6, message = "password required"))]
    pub password: String,
    #[validate(length(min = 1, message = "name required"))]
    pub name: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct LoginRequest {
    #[validate(email, length(min = 6, message = "email required"))]
    pub email: String,
    #[validate(length(min = 6, message = "password required"))]
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "refresh token required"))]
    pub refresh_token: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct UpdateUserRequest {
    #[validate(email, length(min = 6, message = "email required"))]
    pub email: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct UpdateUserURL {
    pub uuid: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserResponse {
    pub user: UserResponseWithoutPassword,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

#[derive(Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserResponseWithoutPassword {
    pub uuid: String,
    pub email: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub two_factor_enabled: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub uuid: String,
    pub email: String,
    pub name: String,
    pub password: String,
    pub created_at: String,
    pub updated_at: String,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
    pub backup_codes: Option<Vec<String>>,
}

impl User {
    pub fn new(
        uuid: String,
        email: String,
        name: String,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    ) -> Self {
        Self {
            uuid,
            email,
            name,
            password: String::new(),
            created_at: created_at.to_string(),
            updated_at: updated_at.to_string(),
            two_factor_enabled: false,
            two_factor_secret: None,
            backup_codes: None,
        }
    }
}

impl From<User> for UserResponseWithoutPassword {
    fn from(user: User) -> Self {
        UserResponseWithoutPassword {
            uuid: user.uuid,
            email: user.email,
            name: user.name,
            created_at: user.created_at,
            updated_at: user.updated_at,
            two_factor_enabled: user.two_factor_enabled,
        }
    }
}

#[derive(Validate, Deserialize, Serialize)]
pub struct Enable2FARequest {
    #[validate(length(min = 6, message = "password required"))]
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct Enable2FAResponse {
    pub secret: String,
    pub qr_code: String,
    pub message: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct Verify2FARequest {
    #[validate(length(min = 6, message = "code required"))]
    pub code: String,
}

#[derive(Deserialize, Serialize)]
pub struct Verify2FAResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct Disable2FARequest {
    #[validate(length(min = 6, message = "password required"))]
    pub password: String,
    #[validate(length(min = 6, message = "code required"))]
    pub code: String,
}

#[derive(Deserialize, Serialize)]
pub struct GenerateBackupCodesResponse {
    pub backup_codes: Vec<String>,
    pub message: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct VerifyBackupCodeRequest {
    pub backup_code: String,
}

#[derive(Deserialize, Serialize)]
pub struct UseBackupCodeForLoginRequest {
    pub email: String,
    pub password: String,
    pub backup_code: String,
}
