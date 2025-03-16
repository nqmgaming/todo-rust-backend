use crate::db::database::Database;
use crate::error::user_error::UserError;
use crate::models::user::{CreateUserRequest, User};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;

#[async_trait]
pub trait UserData {
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, UserError>;
    async fn get_user_by_id(&self, uuid: &str) -> Result<Option<User>, UserError>;
    async fn create_user(&self, uuid: &str, user: &CreateUserRequest) -> Result<(), UserError>;
    async fn update_user(&self, uuid: &str, email: &str) -> Result<Option<User>, UserError>;
}

#[async_trait]
impl UserData for Database {
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, UserError> {
        let query = "SELECT uuid, email, name, password, created_at::TEXT as created_at, updated_at::TEXT as updated_at FROM users WHERE email = $1";

        match sqlx::query(query)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Ok(Some(User {
                uuid: row.get("uuid"),
                email: row.get("email"),
                name: row.get("name"),
                password: row.get("password"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })),
            Ok(None) => Ok(None),
            Err(e) => {
                eprintln!("Error getting user by email: {:?}", e);
                Err(UserError::NoSuchUserFound)
            }
        }
    }

    async fn get_user_by_id(&self, uuid: &str) -> Result<Option<User>, UserError> {
        let query = "SELECT uuid, email, name, password, created_at::TEXT as created_at, updated_at::TEXT as updated_at FROM users WHERE uuid = $1";

        match sqlx::query(query)
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Ok(Some(User {
                uuid: row.get("uuid"),
                email: row.get("email"),
                name: row.get("name"),
                password: row.get("password"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })),
            Ok(None) => Ok(None),
            Err(e) => {
                eprintln!("Error getting user by id: {:?}", e);
                Err(UserError::NoSuchUserFound)
            }
        }
    }

    async fn create_user(&self, uuid: &str, user: &CreateUserRequest) -> Result<(), UserError> {
        // Check if user already exists
        let check_query = "SELECT uuid FROM users WHERE email = $1";

        let existing_user = sqlx::query(check_query)
            .bind(&user.email)
            .fetch_optional(&self.pool)
            .await;

        match existing_user {
            Ok(Some(_)) => Err(UserError::UserAlreadyExists),
            Ok(None) => {
                let now = Utc::now();

                // Insert new user
                let insert_query = "INSERT INTO users (uuid, email, name, password, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)";

                match sqlx::query(insert_query)
                    .bind(uuid)
                    .bind(&user.email)
                    .bind(&user.name)
                    .bind(&user.password)
                    .bind(now)
                    .bind(now)
                    .execute(&self.pool)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        eprintln!("Error adding user: {:?}", e);
                        Err(UserError::UserCreationFailure)
                    }
                }
            }
            Err(e) => {
                eprintln!("Error checking existing user: {:?}", e);
                Err(UserError::UserCreationFailure)
            }
        }
    }

    async fn update_user(&self, uuid: &str, email: &str) -> Result<Option<User>, UserError> {
        let now = Utc::now();
        let query = "UPDATE users SET email = $1, updated_at = $2 WHERE uuid = $3 RETURNING uuid, email, name, password, created_at::TEXT as created_at, updated_at::TEXT as updated_at";

        match sqlx::query(query)
            .bind(email)
            .bind(now)
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Ok(Some(User {
                uuid: row.get("uuid"),
                email: row.get("email"),
                name: row.get("name"),
                password: row.get("password"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })),
            Ok(None) => Ok(None),
            Err(e) => {
                eprintln!("Error updating user: {:?}", e);
                Err(UserError::NoSuchUserFound)
            }
        }
    }
}
