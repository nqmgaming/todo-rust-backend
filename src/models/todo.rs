use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
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

#[derive(Deserialize, Serialize)]
pub struct TodoResponse {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub is_completed: bool,
    pub user_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, Serialize)]
pub struct TodoResponseList {
    pub todos: Vec<TodoResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PaginationParams {
    pub page: Option<i64>,
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TodoFilter {
    pub search: Option<String>,
    pub is_completed: Option<bool>,
    pub sort_by: Option<String>,
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

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize)]
pub struct DeleteTodoResponse {
    pub success: bool,
    pub message: String,
    pub todo_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
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
