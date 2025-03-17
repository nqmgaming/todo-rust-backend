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
        crate::routers::user::register,
        crate::routers::user::login,
        crate::routers::user::refresh_token_endpoint,
        crate::routers::user::update_user,
        crate::routers::user::enable_2fa,
        crate::routers::user::verify_2fa,
        crate::routers::user::disable_2fa,
        crate::routers::user::generate_backup_codes,
        crate::routers::user::login_with_backup_code,
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
            crate::models::user::RefreshTokenRequest,
            crate::models::user::TokenResponse,
            crate::models::user::UpdateUserRequest,
            crate::models::user::UserResponse,
            crate::models::user::User,
            crate::models::user::Enable2FARequest,
            crate::models::user::Enable2FAResponse,
            crate::models::user::Verify2FARequest,
            crate::models::user::Verify2FAResponse,
            crate::models::user::Disable2FARequest,
            crate::models::user::GenerateBackupCodesResponse,
            crate::models::user::VerifyBackupCodeRequest,
            crate::models::user::UseBackupCodeForLoginRequest,
            crate::routers::user::Claims
        ),
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "todos", description = "Todo management endpoints"),
        (name = "users", description = "User management endpoints"),
    ),
    security(
        (),
        ("bearer_auth" = [])
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
            email = "support@quickmem.app",
            url = "https://quickmem.app"
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
