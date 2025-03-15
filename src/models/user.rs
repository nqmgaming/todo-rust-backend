use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct CreateUserRequest {
    #[validate(length(min = 6, message = "username required"))]
    #[schema(example = "johndoe")]
    pub username: String,
    #[validate(length(min = 6, message = "password required"))]
    #[schema(example = "password123")]
    pub password: String,
}

#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 6, message = "username required"))]
    #[schema(example = "johndoe")]
    pub username: String,
    #[validate(length(min = 6, message = "password required"))]
    #[schema(example = "password123")]
    pub password: String,
}

#[derive(Validate, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(length(min = 6, message = "username required"))]
    #[schema(example = "johndoe_updated")]
    pub username: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct UpdateUserURL {
    pub uuid: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UserResponse {
    #[schema(example = "johndoe")]
    pub username: String,
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    #[schema(example = "2023-01-01T12:00:00Z")]
    pub created_at: String,
    #[schema(example = "2023-01-01T12:00:00Z")]
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct User {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub uuid: String,
    #[schema(example = "johndoe")]
    pub username: String,
    #[schema(example = "$2b$12$1234567890123456789012")]
    pub password: String,
    #[schema(example = "2023-01-01T12:00:00Z")]
    pub created_at: String,
    #[schema(example = "2023-01-01T12:00:00Z")]
    pub updated_at: String,
}

impl User {
    pub fn new(
        uuid: String,
        username: String,
        password: String,
        created_at: String,
        updated_at: String,
    ) -> User {
        User {
            uuid,
            username,
            password,
            created_at,
            updated_at,
        }
    }
}
