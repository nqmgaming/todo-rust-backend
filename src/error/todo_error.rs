use actix_web::body::BoxBody;
use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use derive_more::Display;
use serde_json::json;

#[derive(Debug, Display)]
pub enum TodoError {
    #[display("No todos found")]
    NoTodosFound,
    #[display("Todo creation failed")]
    TodoCreationFailure,
    #[display("Todo not found")]
    NoSuchTodoFound,
    #[display("Todo update failed")]
    TodoUpdateFailure,
    #[display("Todo deletion failed")]
    TodoDeletionFailure,
}

impl ResponseError for TodoError {
    fn status_code(&self) -> StatusCode {
        match self {
            TodoError::NoTodosFound => StatusCode::NOT_FOUND,
            TodoError::TodoCreationFailure => StatusCode::INTERNAL_SERVER_ERROR,
            TodoError::NoSuchTodoFound => StatusCode::NOT_FOUND,
            TodoError::TodoUpdateFailure => StatusCode::INTERNAL_SERVER_ERROR,
            TodoError::TodoDeletionFailure => StatusCode::INTERNAL_SERVER_ERROR,
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
