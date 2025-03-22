use crate::services::cache_service::CacheService;
use async_trait::async_trait;
use log::info;
use redis::{aio::ConnectionManager, Client, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RedisClient {
    pub client: Client,
    connection_manager: Arc<Mutex<Option<ConnectionManager>>>,
}

impl RedisClient {
    pub fn new(redis_url: &str) -> Self {
        let client = Client::open(redis_url).expect("Failed to create Redis client");
        info!("Connecting to Redis at {}", redis_url);
        Self {
            client,
            connection_manager: Arc::new(Mutex::new(None)),
        }
    }

    async fn get_conn(&self) -> Result<ConnectionManager, RedisError> {
        let mut manager = self.connection_manager.lock().await;

        if manager.is_none() {
            *manager = Some(ConnectionManager::new(self.client.clone()).await?);
        }

        Ok(manager.as_ref().unwrap().clone())
    }

    pub async fn store_token_state(
        &self,
        token_id: &str,
        user_id: &str,
        ttl_seconds: u64,
    ) -> Result<(), RedisError> {
        let mut conn = self.get_conn().await?;
        let _: () = redis::cmd("SET")
            .arg(token_id)
            .arg(user_id)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn validate_and_invalidate_token(
        &self,
        token_id: &str,
    ) -> Result<Option<String>, RedisError> {
        let mut conn = self.get_conn().await?;

        let user_id: Option<String> = redis::cmd("GET")
            .arg(token_id)
            .query_async(&mut conn)
            .await?;

        if user_id.is_some() {
            let _: () = redis::cmd("DEL")
                .arg(token_id)
                .query_async(&mut conn)
                .await?;
        }

        Ok(user_id)
    }

    pub async fn check_connection(&self) -> Result<(), RedisError> {
        let mut conn = self.get_conn().await?;
        let _: () = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(())
    }
}

#[async_trait]
impl CacheService for RedisClient {
    async fn get_cached<T>(&self, key: &str) -> Result<Option<T>, RedisError>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let mut conn = self.get_conn().await?;
        let data: Option<String> = redis::cmd("GET").arg(key).query_async(&mut conn).await?;

        match data {
            Some(cached_data) => match serde_json::from_str(&cached_data) {
                Ok(parsed) => Ok(Some(parsed)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    async fn set_cached<T>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<(), RedisError>
    where
        T: Serialize + Send + Sync,
    {
        let mut conn = self.get_conn().await?;
        let serialized = serde_json::to_string(value).map_err(|_| {
            RedisError::from((
                redis::ErrorKind::InvalidClientConfig,
                "Failed to serialize data",
            ))
        })?;

        let _: () = redis::cmd("SET")
            .arg(key)
            .arg(serialized)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }

    async fn delete_cached_by_pattern(&self, pattern: &str) -> Result<u64, RedisError> {
        let mut conn = self.get_conn().await?;

        let mut cursor = 0;
        let mut deleted_count = 0;

        loop {
            let scan_result: (i64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await?;

            cursor = scan_result.0;
            let keys = scan_result.1;

            if !keys.is_empty() {
                let del_count: i64 = redis::cmd("DEL").arg(keys).query_async(&mut conn).await?;

                deleted_count += del_count as u64;
            }

            if cursor == 0 {
                break;
            }
        }

        info!(
            "Deleted {} keys matching pattern: {}",
            deleted_count, pattern
        );
        Ok(deleted_count)
    }
}
