use actix_web::body::BoxBody;
use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use derive_more::Display;
use redis::RedisError;
use serde_json::json;

#[derive(Debug, Display)]
pub enum TodoError {
    #[display("Cache operation failed")]
    CacheError,
}

impl From<RedisError> for TodoError {
    fn from(_: RedisError) -> Self {
        TodoError::CacheError
    }
}

impl ResponseError for TodoError {
    fn status_code(&self) -> StatusCode {
        match self {
            TodoError::CacheError => StatusCode::INTERNAL_SERVER_ERROR,
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
