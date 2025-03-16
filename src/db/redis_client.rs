use log::{error, info};
use redis::{AsyncCommands, Client, RedisError};

pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    pub fn new(redis_url: &str) -> Self {
        info!("Connecting to Redis at {}", redis_url);
        let client = Client::open(redis_url).expect("Failed to create Redis client");
        RedisClient { client }
    }

    pub async fn store_token_state(
        &self,
        token_id: &str,
        user_id: &str,
        expires_in_seconds: usize,
    ) -> Result<(), RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        redis::cmd("SET")
            .arg(format!("token:{}", token_id))
            .arg(user_id)
            .arg("EX")
            .arg(expires_in_seconds)
            .query_async(&mut conn)
            .await
    }

    pub async fn validate_and_invalidate_token(
        &self,
        token_id: &str,
    ) -> Result<Option<String>, RedisError> {
        let mut conn = self.client.get_async_connection().await?;

        let key = format!("token:{}", token_id);
        let user_id: Option<String> = redis::cmd("GET").arg(&key).query_async(&mut conn).await?;

        if let Some(user_id) = user_id.clone() {
            redis::cmd("DEL").arg(&key).query_async(&mut conn).await?;

            info!("Token {} was valid for user {}", token_id, user_id);
        } else {
            info!("Token {} not found or already invalidated", token_id);
        }

        Ok(user_id)
    }

    pub async fn check_connection(&self) -> Result<(), RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;

        if pong == "PONG" {
            info!("Connected to Redis successfully");
            Ok(())
        } else {
            error!("Could not get PONG response from Redis");
            Err(RedisError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not get PONG response from Redis",
            )))
        }
    }
}
