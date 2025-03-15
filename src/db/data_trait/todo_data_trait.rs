use crate::db::database::Database;
use crate::error::AppError;
use crate::models::todo::{CreateTodoRequest, DeleteTodoResponse, PaginationParams, Todo, TodoFilter, TodoResponse, TodoResponseList};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

#[async_trait]
pub trait TodoData {
    async fn get_all_todos(&self, user_id: String, pagination: PaginationParams, filter: TodoFilter) -> Result<TodoResponseList, AppError>;
    async fn get_one_todo(&self, todo_id: String) -> Result<TodoResponse, AppError>;
    async fn add_todo(&self, user_id: String, todo: CreateTodoRequest) -> Result<TodoResponse, AppError>;
    async fn update_todo(
        &self,
        todo_uuid: String,
        title: Option<String>,
        description: Option<String>,
        is_completed: Option<bool>,
    ) -> Result<Todo, AppError>;
    async fn delete_todo(&self, todo_uuid: String) -> Result<DeleteTodoResponse, AppError>;
}

#[async_trait]
impl TodoData for Database {
    async fn get_all_todos(&self, user_id: String, pagination: PaginationParams, filter: TodoFilter) -> Result<TodoResponseList, AppError> {
        let page = pagination.page.unwrap_or(1);
        let page_size = pagination.page_size.unwrap_or(10);
        let offset = (page - 1) * page_size;
        
        // Xây dựng truy vấn với các điều kiện lọc
        let mut count_query = "SELECT COUNT(*) as total FROM todos WHERE owner_id = $1".to_string();
        let mut query = "SELECT uuid, title, description, is_completed, owner_id, created_at, updated_at FROM todos WHERE owner_id = $1".to_string();
        
        let mut params: Vec<String> = vec![user_id.clone()];
        let mut param_index = 2; // Bắt đầu từ $2
        
        // Thêm điều kiện tìm kiếm
        if let Some(search) = filter.search {
            let search_condition = format!(" AND (title ILIKE ${} OR description ILIKE ${})", param_index, param_index);
            count_query.push_str(&search_condition);
            query.push_str(&search_condition);
            params.push(format!("%{}%", search));
            param_index += 1;
        }
        
        // Thêm điều kiện lọc theo trạng thái hoàn thành
        if let Some(is_completed) = filter.is_completed {
            let completed_condition = format!(" AND is_completed = ${}", param_index);
            count_query.push_str(&completed_condition);
            query.push_str(&completed_condition);
            params.push(is_completed.to_string());
            param_index += 1;
        }
        
        // Thêm sắp xếp
        let sort_by = filter.sort_by.unwrap_or_else(|| "created_at".to_string());
        let sort_order = filter.sort_order.unwrap_or_else(|| "desc".to_string());
        
        // Kiểm tra tính hợp lệ của sort_by để tránh SQL injection
        let valid_sort_columns = vec!["created_at", "updated_at", "title", "is_completed"];
        let sort_by = if valid_sort_columns.contains(&sort_by.as_str()) {
            sort_by
        } else {
            "created_at".to_string()
        };
        
        // Kiểm tra tính hợp lệ của sort_order
        let sort_order = if sort_order.to_lowercase() == "asc" {
            "ASC"
        } else {
            "DESC"
        };
        
        query.push_str(&format!(" ORDER BY {} {} LIMIT ${} OFFSET ${}", sort_by, sort_order, param_index, param_index + 1));
        
        // Lấy tổng số todo
        let mut count_query_builder = sqlx::query(&count_query);
        for param in &params {
            count_query_builder = count_query_builder.bind(param);
        }
        
        let total: i64 = count_query_builder
            .fetch_one(&self.pool)
            .await?
            .get("total");
            
        let total_pages = (total + page_size - 1) / page_size; // Làm tròn lên
        
        // Lấy danh sách todo với phân trang và lọc
        let mut query_builder = sqlx::query(&query);
        for param in &params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder = query_builder.bind(page_size).bind(offset);
        
        let rows = query_builder.fetch_all(&self.pool).await?;

        let mut todos = Vec::new();
        for row in rows {
            let created_at: chrono::DateTime<Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<Utc> = row.get("updated_at");
            
            todos.push(TodoResponse {
                uuid: row.get("uuid"),
                title: row.get("title"),
                description: row.get("description"),
                is_completed: row.get("is_completed"),
                user_id: row.get("owner_id"),
                created_at: created_at.to_string(),
                updated_at: updated_at.to_string(),
            });
        }

        Ok(TodoResponseList {
            todos,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    async fn get_one_todo(&self, todo_id: String) -> Result<TodoResponse, AppError> {
        let query = "SELECT uuid, title, description, is_completed, owner_id, created_at, updated_at FROM todos WHERE uuid = $1";
        
        let row = sqlx::query(query)
            .bind(&todo_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                match e {
                    sqlx::Error::RowNotFound => AppError::not_found("Todo not found"),
                    _ => {
                        eprintln!("Error getting todo: {:?}", e);
                        AppError::internal_server_error("Failed to get todo")
                    }
                }
            })?;
        
        let created_at: chrono::DateTime<Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<Utc> = row.get("updated_at");
        
        Ok(TodoResponse {
            uuid: row.get("uuid"),
            title: row.get("title"),
            description: row.get("description"),
            is_completed: row.get("is_completed"),
            user_id: row.get("owner_id"),
            created_at: created_at.to_string(),
            updated_at: updated_at.to_string(),
        })
    }

    async fn add_todo(&self, user_id: String, todo: CreateTodoRequest) -> Result<TodoResponse, AppError> {
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();

        let query = "INSERT INTO todos (uuid, title, description, is_completed, owner_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *";
        
        let row = sqlx::query(query)
            .bind(&uuid)
            .bind(&todo.title)
            .bind(&todo.description)
            .bind(false)
            .bind(&user_id)
            .bind(now)
            .bind(now)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("Error adding todo: {:?}", e);
                AppError::internal_server_error("Failed to add todo")
            })?;
        
        let created_todo = Todo::new(
            row.get("uuid"),
            row.get("title"),
            row.get("description"),
            row.get("is_completed"),
            row.get("owner_id"),
            row.get("created_at"),
            row.get("updated_at"),
        );
        
        Ok(TodoResponse {
            uuid: created_todo.uuid,
            title: created_todo.title,
            description: created_todo.description,
            is_completed: created_todo.is_completed,
            user_id: created_todo.owner_id,
            created_at: created_todo.created_at.to_string(),
            updated_at: created_todo.updated_at.to_string(),
        })
    }

    async fn update_todo(
        &self,
        todo_uuid: String,
        title: Option<String>,
        description: Option<String>,
        is_completed: Option<bool>,
    ) -> Result<Todo, AppError> {
        let existing_todo = self.get_one_todo(todo_uuid.clone()).await?;
        
        let title = title.unwrap_or(existing_todo.title);
        let description = description.unwrap_or(existing_todo.description);
        let is_completed = is_completed.unwrap_or(existing_todo.is_completed);
        let now = Utc::now();
        
        let query = "UPDATE todos SET title = $1, description = $2, is_completed = $3, updated_at = $4 WHERE uuid = $5 RETURNING *";
        
        let row = sqlx::query(query)
            .bind(&title)
            .bind(&description)
            .bind(is_completed)
            .bind(now)
            .bind(&todo_uuid)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("Error updating todo: {:?}", e);
                AppError::internal_server_error("Failed to update todo")
            })?;
        
        Ok(Todo::new(
            row.get("uuid"),
            row.get("title"),
            row.get("description"),
            row.get("is_completed"),
            row.get("owner_id"),
            row.get("created_at"),
            row.get("updated_at"),
        ))
    }

    async fn delete_todo(&self, todo_uuid: String) -> Result<DeleteTodoResponse, AppError> {
        let check_query = "SELECT uuid FROM todos WHERE uuid = $1";
        let todo_exists = sqlx::query(check_query)
            .bind(&todo_uuid)
            .fetch_optional(&self.pool)
            .await?;
            
        if todo_exists.is_none() {
            return Err(AppError::not_found(format!("Todo with id {} not found", todo_uuid)));
        }
        
        let query = "DELETE FROM todos WHERE uuid = $1";
        
        sqlx::query(query)
            .bind(&todo_uuid)
            .execute(&self.pool)
            .await?;

        Ok(DeleteTodoResponse {
            success: true,
            message: "Todo deleted successfully".to_string(),
            todo_id: todo_uuid,
        })
    }
}
