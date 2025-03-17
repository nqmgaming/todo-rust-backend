use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct CreateUserRequest {
    #[validate(email, length(min = 6, message = "email required"))]
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    #[validate(length(min = 6, message = "password required"))]
    #[schema(example = "password123")]
    pub password: String,
    #[validate(length(min = 1, message = "name required"))]
    #[schema(example = "John Doe")]
    pub name: String,
}

#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    #[validate(email, length(min = 6, message = "email required"))]
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    #[validate(length(min = 6, message = "password required"))]
    #[schema(example = "password123")]
    pub password: String,
    #[schema(example = "123456")]
    pub totp_code: Option<String>,
}

#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "refresh token required"))]
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(email, length(min = 6, message = "email required"))]
    #[schema(example = "john.doe.updated@example.com")]
    pub email: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct UpdateUserURL {
    pub uuid: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UserResponse {
    pub user: UserResponseWithoutPassword,
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
    #[schema(example = "Bearer")]
    pub token_type: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct TokenResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
    #[schema(example = "Bearer")]
    pub token_type: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UserResponseWithoutPassword {
    pub uuid: String,
    pub email: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub two_factor_enabled: bool,
}

#[derive(Deserialize, Serialize, ToSchema, Clone)]
pub struct User {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub uuid: String,
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    #[schema(example = "John Doe")]
    pub name: String,
    #[schema(example = "$2b$12$1234567890123456789012")]
    pub password: String,
    #[schema(example = "2023-01-01T12:00:00")]
    pub created_at: String,
    #[schema(example = "2023-01-01T12:00:00")]
    pub updated_at: String,
    #[schema(example = "false")]
    pub two_factor_enabled: bool,
    #[schema(example = "ABCDEFGHIJKLMNOP")]
    pub two_factor_secret: Option<String>,
    #[schema(example = "[\"hash1\", \"hash2\", \"hash3\"]")]
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


#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct Enable2FARequest {
    #[validate(length(min = 6, message = "password required"))]
    #[schema(example = "password123")]
    pub password: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Enable2FAResponse {
    #[schema(example = "ABCDEFGHIJKLMNOP")]
    pub secret: String,
    #[schema(
        example = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAQAAAAEAAQMAAABmvDolAAAABlBMVEX///8AAABVwtN+AAAA"
    )]
    pub qr_code: String,
    #[schema(example = "2FA enabled successfully")]
    pub message: String,
}

#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct Verify2FARequest {
    #[validate(length(min = 6, message = "code required"))]
    #[schema(example = "123456")]
    pub code: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Verify2FAResponse {
    #[schema(example = "true")]
    pub success: bool,
    #[schema(example = "2FA verified successfully")]
    pub message: String,
}

#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct Disable2FARequest {
    #[validate(length(min = 6, message = "password required"))]
    #[schema(example = "password123")]
    pub password: String,
    #[validate(length(min = 6, message = "code required"))]
    #[schema(example = "123456")]
    pub code: String,
}


#[derive(Deserialize, Serialize, ToSchema)]
pub struct GenerateBackupCodesResponse {
    #[schema(example = "[\"abcde-fghij\", \"klmno-pqrst\"]")]
    pub backup_codes: Vec<String>,
    #[schema(example = "Backup codes generated successfully")]
    pub message: String,
}

#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct VerifyBackupCodeRequest {
    #[schema(example = "abcdefghij")]
    pub backup_code: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UseBackupCodeForLoginRequest {
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    #[schema(example = "password123")]
    pub password: String,
    #[schema(example = "abcdefghij")]
    pub backup_code: String,
}
