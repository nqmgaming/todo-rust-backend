use crate::models::todo::ApiResponse;
use actix_web::body::BoxBody;
use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use std::fmt;

#[derive(Debug)]
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
}

impl AppError {
    pub fn new(status_code: StatusCode, message: impl Into<String>) -> Self {
        AppError {
            status_code,
            message: message.into(),
        }
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let error_response = ApiResponse::<()> {
            success: false,
            message: self.message.clone(),
            data: None,
        };

        HttpResponse::build(self.status_code)
            .insert_header(ContentType::json())
            .json(error_response)
    }
}

// Implement From for common error types
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::not_found("Resource not found"),
            sqlx::Error::Database(db_error) => {
                if let Some(code) = db_error.code() {
                    if code == "23503" {
                        // Foreign key violation
                        return Self::bad_request(format!(
                            "Foreign key constraint violation: {}",
                            db_error.message()
                        ));
                    }
                    if code == "23505" {
                        // Unique violation
                        return Self::bad_request(format!(
                            "Unique constraint violation: {}",
                            db_error.message()
                        ));
                    }
                }
                Self::internal_server_error(format!("Database error: {}", db_error.message()))
            }
            _ => Self::internal_server_error(format!("Database error: {}", error)),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::internal_server_error(format!("IO error: {}", error))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        Self::unauthorized(format!("JWT error: {}", error))
    }
}
