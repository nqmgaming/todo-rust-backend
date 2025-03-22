use crate::models::todo::{DeleteTodoResponse, TodoResponse, TodoResponseList};
use crate::routers::health::HealthResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiResponseTodoResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<TodoResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponseTodoResponseList {
    pub success: bool,
    pub message: String,
    pub data: Option<TodoResponseList>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponseDeleteTodoResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<DeleteTodoResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponseHealthResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<HealthResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponseEmpty {
    pub success: bool,
    pub message: String,
    pub data: Option<()>,
}
