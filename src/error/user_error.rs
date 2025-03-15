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
}

impl ResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserError::UserCreationFailure => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::NoSuchUserFound => StatusCode::NOT_FOUND,
            UserError::AuthenticationFailure => StatusCode::UNAUTHORIZED,
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