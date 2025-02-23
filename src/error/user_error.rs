use actix_web::body::BoxBody;
use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use derive_more::Display;

#[derive(Debug, Display)]
pub enum UserError {
    NoUsersFound,
    UserCreationFailure,
    NoSuchUserFound,
}

impl ResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserError::NoUsersFound => StatusCode::NOT_FOUND,
            UserError::UserCreationFailure => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::NoSuchUserFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}