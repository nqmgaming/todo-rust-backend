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

    async fn delete_cached(&self, key: &str) -> Result<(), RedisError>;

    async fn delete_cached_by_pattern(&self, pattern: &str) -> Result<u64, RedisError>;
}
