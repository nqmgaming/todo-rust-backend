use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize, Serialize)]
pub struct CreateUserRequest {
    #[validate(length(min = 6, message = "username required"))]
    pub username: String,
    #[validate(length(min = 6, message = "password required"))]
    pub password: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct CheckEmailRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct UpdateUserRequest {
    #[validate(length(min = 6, message = "username required"))]
    pub username: String,
}
#[derive(Validate, Deserialize, Serialize)]
pub struct UpdateUserURL {
    pub uuid: String,
}
#[derive(Deserialize, Serialize)]
pub struct UserResponse {
    pub username: String,
    pub access_token: String,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Deserialize, Serialize)]
pub struct CheckEmailOutputResponse {
    pub input: String,
    pub is_reachable: bool,
    pub misc: String,
    pub mx: String,
    pub smtp: String,
    pub syntax: String,
}
#[derive(Deserialize, Serialize)]
pub struct User {
    pub uuid: String,
    pub username: String,
    pub password: String,
    pub created_at: String,
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
