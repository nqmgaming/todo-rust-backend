use actix_web::body::BoxBody;
use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use derive_more::Display;
use serde_json::json;

#[derive(Debug, Display)]
pub enum UserError {
    #[display("User creation failed")]
    UserCreationFailure,
    #[display("No such user found")]
    NoSuchUserFound,
    #[display("Authentication failed")]
    AuthenticationFailure,
    #[display("Username already exists")]
    UserAlreadyExists,
    #[display("Validation error: {}", _0)]
    ValidationError(String),
    #[display("Invalid refresh token")]
    InvalidRefreshToken,
    #[display("Token creation failed")]
    TokenCreationFailure,
    #[display("Password hashing failed")]
    PasswordHashingFailure,
    #[display("User not found")]
    UserNotFound,
    #[display("Invalid credentials")]
    InvalidCredentials,
}

impl ResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserError::UserCreationFailure => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::NoSuchUserFound => StatusCode::NOT_FOUND,
            UserError::AuthenticationFailure => StatusCode::UNAUTHORIZED,
            UserError::UserAlreadyExists => StatusCode::CONFLICT,
            UserError::ValidationError(_) => StatusCode::BAD_REQUEST,
            UserError::InvalidRefreshToken => StatusCode::UNAUTHORIZED,
            UserError::TokenCreationFailure => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::PasswordHashingFailure => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::UserNotFound => StatusCode::NOT_FOUND,
            UserError::InvalidCredentials => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let error_json = json!({
            "status": "error",
            "code": self.status_code().as_u16(),
            "message": self.to_string()
        });

        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(error_json)
    }
}
