use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use log::{info, warn, error};
use crate::error::AppError;

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
        // Create tables if they don't exist
        let setup_sql = include_str!("../../setup_db.sql");
        match sqlx::query(setup_sql)
            .execute(&pool)
            .await {
                Ok(_) => info!("Database tables created successfully"),
                Err(e) => {
                    warn!("Error executing setup script as a single transaction: {:?}", e);
                    info!("Trying to execute statements separately...");
                    // Try to execute each statement separately
                    let statements = setup_sql.split(";").filter(|s| !s.trim().is_empty());
                    for stmt in statements {
                        if let Err(e) = sqlx::query(&format!("{};", stmt.trim()))
                            .execute(&pool)
                            .await {
                                error!("Error executing statement: {:?}\nStatement: {}", e, stmt);
                            } else {
                                info!("Statement executed successfully");
                            }
                    }
                }
            }
        
        Ok(Database { pool })
    }
}
