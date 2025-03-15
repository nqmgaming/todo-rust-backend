use crate::db::database::Database;
use crate::models::user::User;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;

#[async_trait]
pub trait UserData {
    async fn add_user(&self, user: User) -> Option<User>;
    async fn update_user(&self, uuid: String, username: String) -> Option<User>;
}

#[async_trait]
impl UserData for Database {
    async fn add_user(&self, user: User) -> Option<User> {
        // Check if user already exists
        let check_query = "SELECT uuid FROM users WHERE username = $1";

        let existing_user = sqlx::query(check_query)
            .bind(&user.username)
            .fetch_optional(&self.pool)
            .await;

        match existing_user {
            Ok(Some(_)) => {
                eprintln!("User already exists");
                None
            }
            Ok(None) => {
                // Parse created_at and updated_at as timestamps
                let created_at = match chrono::DateTime::parse_from_rfc3339(&user.created_at) {
                    Ok(dt) => dt.with_timezone(&Utc),
                    Err(_) => Utc::now(),
                };

                let updated_at = match chrono::DateTime::parse_from_rfc3339(&user.updated_at) {
                    Ok(dt) => dt.with_timezone(&Utc),
                    Err(_) => Utc::now(),
                };

                // Insert new user
                let insert_query = "INSERT INTO users (uuid, username, password, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) RETURNING uuid";

                match sqlx::query(insert_query)
                    .bind(&user.uuid)
                    .bind(&user.username)
                    .bind(&user.password)
                    .bind(created_at)
                    .bind(updated_at)
                    .fetch_one(&self.pool)
                    .await
                {
                    Ok(_) => Some(user),
                    Err(e) => {
                        eprintln!("Error adding user: {:?}", e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Error checking existing user: {:?}", e);
                None
            }
        }
    }

    async fn update_user(&self, uuid: String, username: String) -> Option<User> {
        let now = Utc::now();
        let query = "UPDATE users SET username = $1, updated_at = $2 WHERE uuid = $3 RETURNING uuid, username, password, created_at::TEXT as created_at, updated_at::TEXT as updated_at";

        match sqlx::query(query)
            .bind(&username)
            .bind(now)
            .bind(&uuid)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Some(User {
                uuid: row.get("uuid"),
                username: row.get("username"),
                password: row.get("password"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }),
            Ok(None) => None,
            Err(e) => {
                eprintln!("Error updating user: {:?}", e);
                None
            }
        }
    }
}
