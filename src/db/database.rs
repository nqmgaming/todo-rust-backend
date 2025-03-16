use crate::error::AppError;
use log::{error, info};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn init() -> Result<Self, AppError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::internal_server_error("DATABASE_URL must be set"))?;

        info!("Connecting to database...");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .map_err(|e| {
                error!("Failed to create database pool: {}", e);
                AppError::internal_server_error(format!("Failed to create pool: {}", e))
            })?;

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

        Ok(Database { pool })
    }
}
