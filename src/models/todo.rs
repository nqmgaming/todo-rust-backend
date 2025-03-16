use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateTodoRequest {
    #[schema(example = "Học Rust")]
    pub title: String,
    #[schema(example = "Học ngôn ngữ lập trình Rust trong 30 ngày")]
    pub description: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateTodoRequest {
    #[schema(example = "Học Rust nâng cao")]
    pub title: Option<String>,
    #[schema(example = "Học ngôn ngữ lập trình Rust nâng cao trong 60 ngày")]
    pub description: Option<String>,
    #[schema(example = "true")]
    pub is_completed: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTodoURL {
    pub uuid: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetTodoURL {
    pub uuid: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct TodoResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub uuid: String,
    #[schema(example = "Học Rust")]
    pub title: String,
    #[schema(example = "Học ngôn ngữ lập trình Rust trong 30 ngày")]
    pub description: String,
    #[schema(example = "false")]
    pub is_completed: bool,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: String,
    #[schema(example = "2023-01-01T12:00:00Z")]
    pub created_at: String,
    #[schema(example = "2023-01-01T12:00:00Z")]
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct TodoResponseList {
    pub todos: Vec<TodoResponse>,
    #[schema(example = "100")]
    pub total: i64,
    #[schema(example = "1")]
    pub page: i64,
    #[schema(example = "10")]
    pub page_size: i64,
    #[schema(example = "10")]
    pub total_pages: i64,
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct PaginationParams {
    #[schema(example = "1")]
    pub page: Option<i64>,
    #[schema(example = "10")]
    pub page_size: Option<i64>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(10),
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct TodoFilter {
    #[schema(example = "Rust")]
    pub search: Option<String>,
    #[schema(example = "false")]
    pub is_completed: Option<bool>,
    #[schema(example = "created_at")]
    pub sort_by: Option<String>,
    #[schema(example = "desc")]
    pub sort_order: Option<String>,
}

impl Default for TodoFilter {
    fn default() -> Self {
        Self {
            search: None,
            is_completed: None,
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct TodoQueryParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    #[serde(flatten)]
    pub filter: TodoFilter,
}

impl Default for TodoQueryParams {
    fn default() -> Self {
        Self {
            pagination: PaginationParams::default(),
            filter: TodoFilter::default(),
        }
    }
}

impl std::fmt::Display for TodoQueryParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "page={};page_size={};search={};is_completed={};sort_by={};sort_order={}",
            self.pagination.page.unwrap_or(1),
            self.pagination.page_size.unwrap_or(10),
            self.filter.search.as_deref().unwrap_or(""),
            self.filter.is_completed.unwrap_or(false),
            self.filter.sort_by.as_deref().unwrap_or("created_at"),
            self.filter.sort_order.as_deref().unwrap_or("desc")
        )
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DeleteTodoResponse {
    #[schema(example = "true")]
    pub success: bool,
    #[schema(example = "Todo đã được xóa thành công")]
    pub message: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub todo_id: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    #[schema(example = "true")]
    pub success: bool,
    #[schema(example = "Thao tác thành công")]
    pub message: String,
    pub data: Option<T>,
}

#[derive(Deserialize, Serialize)]
pub struct Todo {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub is_completed: bool,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Todo {
    pub fn new(
        uuid: String,
        title: String,
        description: String,
        is_completed: bool,
        owner_id: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Todo {
        Todo {
            uuid,
            title,
            description,
            is_completed,
            owner_id,
            created_at,
            updated_at,
        }
    }
}

impl From<Todo> for TodoResponse {
    fn from(todo: Todo) -> Self {
        TodoResponse {
            uuid: todo.uuid,
            title: todo.title,
            description: todo.description,
            is_completed: todo.is_completed,
            user_id: todo.owner_id,
            created_at: todo.created_at.to_string(),
            updated_at: todo.updated_at.to_string(),
        }
    }
}
