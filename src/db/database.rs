use crate::db::redis_client::RedisClient;
use log::error;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

pub struct Database {
    pub pool: PgPool,
    pub redis_client: RedisClient,
}

impl Database {
    pub async fn init() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres");

        let redis_client = RedisClient::new(&redis_url);

        if let Err(e) = redis_client.check_connection().await {
            error!("Failed to connect to Redis: {}", e);
        }

        Self { pool, redis_client }
    }
}
