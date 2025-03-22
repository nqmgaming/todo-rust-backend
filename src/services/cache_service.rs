use async_trait::async_trait;
use redis::RedisError;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait CacheService {
    async fn get_cached<T>(&self, key: &str) -> Result<Option<T>, RedisError>
    where
        T: DeserializeOwned + Send + Sync;

    async fn set_cached<T>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<(), RedisError>
    where
        T: Serialize + Send + Sync;

    async fn delete_cached_by_pattern(&self, pattern: &str) -> Result<u64, RedisError>;
    async fn set_with_expiry(
        &self,
        key: &str,
        value: &str,
        expiry_seconds: u64,
    ) -> Result<(), redis::RedisError>;
    async fn get(&self, key: &str) -> Result<String, redis::RedisError>;
    async fn del(&self, key: &str) -> Result<(), redis::RedisError>;
}
