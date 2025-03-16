use crate::db::data_trait::todo_data_trait::TodoData;
use crate::db::data_trait::user_data_trait::UserData;
use crate::db::redis_client::RedisClient;
use crate::error::todo_error::TodoError;
use crate::error::user_error::UserError;
use crate::error::AppError;
use crate::models::todo::{CreateTodoRequest, Todo, UpdateTodoRequest};
use crate::models::user::{CreateUserRequest, UpdateUserRequest, User};
use log::{error, info};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;
use std::time::Duration;

pub struct Database {
    pub pool: Pool<Postgres>,
    pub redis_client: RedisClient,
}

impl Database {
    pub async fn init() -> Result<Self, AppError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::internal_server_error("DATABASE_URL must be set"))?;
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        info!("Connecting to database...");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .map_err(|e| {
                error!("Failed to create database pool: {}", e);
                AppError::internal_server_error(format!("Failed to create pool: {}", e))
            })?;

        info!("Connecting to Redis...");
        let redis_client = RedisClient::new(&redis_url);

        if let Err(e) = redis_client.check_connection().await {
            error!("Failed to connect to Redis: {}", e);
            // Không trả về lỗi, chỉ log để ứng dụng vẫn có thể chạy nếu Redis không khả dụng
        }

        info!("Running database setup script...");
        let setup_sql = include_str!("../../setup_db.sql");

        // Split the SQL script into individual statements based on semicolons
        let statements = setup_sql.split(';').filter(|s| !s.trim().is_empty());

        // Execute each statement separately
        for stmt in statements {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }

            match sqlx::query(stmt).execute(&pool).await {
                Ok(_) => {
                    info!("Statement executed successfully: {}", stmt);
                }
                Err(e) => {
                    error!("Error executing statement: {:?}\nStatement: {}", e, stmt);
                }
            }
        }

        Ok(Database { pool, redis_client })
    }
}
