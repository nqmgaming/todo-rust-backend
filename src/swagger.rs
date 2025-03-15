use crate::models::todo::{DeleteTodoResponse, TodoResponse, TodoResponseList};
use crate::routers::health::HealthResponse;
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routers::health::health,
        crate::routers::todo::get_todos,
        crate::routers::todo::get_todo,
        crate::routers::todo::create_todo,
        crate::routers::todo::update_todo,
        crate::routers::todo::delete_todo,
        crate::routers::user::create_user,
        crate::routers::user::login_user,
        crate::routers::user::update_user,
    ),
    components(
        schemas(
            ApiResponseTodoResponse,
            ApiResponseTodoResponseList,
            ApiResponseDeleteTodoResponse,
            ApiResponseHealthResponse,
            ApiResponseEmpty,
            crate::models::todo::CreateTodoRequest,
            crate::models::todo::UpdateTodoRequest,
            crate::models::todo::TodoResponse,
            crate::models::todo::TodoResponseList,
            crate::models::todo::DeleteTodoResponse,
            crate::models::todo::TodoQueryParams,
            crate::models::todo::PaginationParams,
            crate::models::todo::TodoFilter,
            crate::routers::health::HealthResponse,
            crate::models::user::CreateUserRequest,
            crate::models::user::LoginRequest,
            crate::models::user::UpdateUserRequest,
            crate::models::user::UserResponse,
            crate::models::user::User,
            crate::routers::user::Claims
        ),
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "todos", description = "Todo management endpoints"),
        (name = "users", description = "User management endpoints"),
    ),
    info(
        title = "Rust Backend API",
        version = "1.0.0",
        description = "A RESTful API built with Rust and Actix Web",
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        ),
        contact(
            name = "API Support",
            email = "support@example.com",
            url = "https://example.com/support"
        )
    )
)]
pub struct ApiDoc;

// Định nghĩa các kiểu cụ thể cho ApiResponse thay vì sử dụng generic
#[derive(utoipa::ToSchema, Serialize, Deserialize)]
pub struct ApiResponseTodoResponse {
    #[schema(example = "true")]
    pub success: bool,
    #[schema(example = "Thao tác thành công")]
    pub message: String,
    pub data: Option<TodoResponse>,
}

#[derive(utoipa::ToSchema, Serialize, Deserialize)]
pub struct ApiResponseTodoResponseList {
    #[schema(example = "true")]
    pub success: bool,
    #[schema(example = "Thao tác thành công")]
    pub message: String,
    pub data: Option<TodoResponseList>,
}

#[derive(utoipa::ToSchema, Serialize, Deserialize)]
pub struct ApiResponseDeleteTodoResponse {
    #[schema(example = "true")]
    pub success: bool,
    #[schema(example = "Thao tác thành công")]
    pub message: String,
    pub data: Option<DeleteTodoResponse>,
}

#[derive(utoipa::ToSchema, Serialize, Deserialize)]
pub struct ApiResponseHealthResponse {
    #[schema(example = "true")]
    pub success: bool,
    #[schema(example = "Thao tác thành công")]
    pub message: String,
    pub data: Option<HealthResponse>,
}

#[derive(utoipa::ToSchema, Serialize, Deserialize)]
pub struct ApiResponseEmpty {
    #[schema(example = "true")]
    pub success: bool,
    #[schema(example = "Thao tác thành công")]
    pub message: String,
    pub data: Option<()>,
}
