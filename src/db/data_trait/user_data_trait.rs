use crate::db::database::Database;
use crate::error::user_error::UserError;
use crate::models::user::{CreateUserRequest, User};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;

#[async_trait]
pub trait UserData {
    async fn get_user_by_email(&self, email: &str) -> Result<User, UserError>;
    async fn get_user_by_uuid(&self, uuid: &str) -> Result<User, UserError>;
    async fn create_user(&self, uuid: &str, user: &CreateUserRequest) -> Result<User, UserError>;
    async fn update_user(&self, user: &User) -> Result<User, UserError>;
    async fn enable_2fa(&self, uuid: &str, secret: &str) -> Result<(), UserError>;
    async fn verify_2fa(&self, uuid: &str) -> Result<(), UserError>;
    async fn disable_2fa(&self, uuid: &str) -> Result<(), UserError>;
}

#[async_trait]
impl UserData for Database {
    async fn get_user_by_email(&self, email: &str) -> Result<User, UserError> {
        let query = "SELECT uuid, email, name, password, created_at::TEXT as created_at, updated_at::TEXT as updated_at, two_factor_enabled, two_factor_secret, backup_codes FROM users WHERE email = $1";

        match sqlx::query(query)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Ok(User {
                uuid: row.get("uuid"),
                email: row.get("email"),
                name: row.get("name"),
                password: row.get("password"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                two_factor_enabled: row.get("two_factor_enabled"),
                two_factor_secret: row.get("two_factor_secret"),
                backup_codes: row.get("backup_codes"),
            }),
            Ok(None) => Err(UserError::NoSuchUserFound),
            Err(e) => {
                eprintln!("Error getting user by email: {:?}", e);
                Err(UserError::DatabaseError(e.to_string()))
            }
        }
    }

    async fn get_user_by_uuid(&self, uuid: &str) -> Result<User, UserError> {
        let query = "SELECT uuid, email, name, password, created_at::TEXT as created_at, updated_at::TEXT as updated_at, two_factor_enabled, two_factor_secret, backup_codes FROM users WHERE uuid = $1";

        match sqlx::query(query)
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Ok(User {
                uuid: row.get("uuid"),
                email: row.get("email"),
                name: row.get("name"),
                password: row.get("password"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                two_factor_enabled: row.get("two_factor_enabled"),
                two_factor_secret: row.get("two_factor_secret"),
                backup_codes: row.get("backup_codes"),
            }),
            Ok(None) => Err(UserError::NoSuchUserFound),
            Err(e) => {
                eprintln!("Error getting user by uuid: {:?}", e);
                Err(UserError::DatabaseError(e.to_string()))
            }
        }
    }

    async fn create_user(&self, uuid: &str, user: &CreateUserRequest) -> Result<User, UserError> {
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
                let insert_query = "INSERT INTO users (uuid, email, name, password, created_at, updated_at, two_factor_enabled) VALUES ($1, $2, $3, $4, $5, $6, $7)";

                match sqlx::query(insert_query)
                    .bind(&uuid)
                    .bind(&user.email)
                    .bind(&user.name)
                    .bind(&user.password)
                    .bind(now)
                    .bind(now)
                    .bind(false)
                    .execute(&self.pool)
                    .await
                {
                    Ok(_) => Ok(User {
                        uuid: uuid.to_string(),
                        email: user.email.clone(),
                        name: user.name.clone(),
                        password: user.password.clone(),
                        created_at: now.to_string(),
                        updated_at: now.to_string(),
                        two_factor_enabled: false,
                        two_factor_secret: None,
                        backup_codes: None,
                    }),
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

    async fn update_user(&self, user: &User) -> Result<User, UserError> {
        let query = "UPDATE users SET email = $1, name = $2, password = $3, updated_at = NOW(), two_factor_enabled = $4, two_factor_secret = $5, backup_codes = $6 WHERE uuid = $7 RETURNING uuid, email, name, password, created_at::TEXT as created_at, updated_at::TEXT as updated_at, two_factor_enabled, two_factor_secret, backup_codes";

        match sqlx::query(query)
            .bind(&user.email)
            .bind(&user.name)
            .bind(&user.password)
            .bind(user.two_factor_enabled)
            .bind(&user.two_factor_secret)
            .bind(&user.backup_codes)
            .bind(&user.uuid)
            .fetch_one(&self.pool)
            .await
        {
            Ok(row) => Ok(User {
                uuid: row.get("uuid"),
                email: row.get("email"),
                name: row.get("name"),
                password: row.get("password"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                two_factor_enabled: row.get("two_factor_enabled"),
                two_factor_secret: row.get("two_factor_secret"),
                backup_codes: row.get("backup_codes"),
            }),
            Err(e) => {
                eprintln!("Error updating user: {:?}", e);
                Err(UserError::DatabaseError(e.to_string()))
            }
        }
    }

    async fn enable_2fa(&self, uuid: &str, secret: &str) -> Result<(), UserError> {
        let now = Utc::now();
        let query = "UPDATE users SET two_factor_secret = $1, two_factor_enabled = $2, updated_at = $3 WHERE uuid = $4";

        match sqlx::query(query)
            .bind(secret)
            .bind(true)
            .bind(now)
            .bind(uuid)
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error enabling 2FA: {:?}", e);
                Err(UserError::NoSuchUserFound)
            }
        }
    }

    async fn verify_2fa(&self, uuid: &str) -> Result<(), UserError> {
        let now = Utc::now();
        let query = "UPDATE users SET two_factor_enabled = $1, updated_at = $2 WHERE uuid = $3";

        match sqlx::query(query)
            .bind(true)
            .bind(now)
            .bind(uuid)
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error verifying 2FA: {:?}", e);
                Err(UserError::NoSuchUserFound)
            }
        }
    }

    async fn disable_2fa(&self, uuid: &str) -> Result<(), UserError> {
        let now = Utc::now();
        let query = "UPDATE users SET two_factor_secret = NULL, two_factor_enabled = $1, updated_at = $2 WHERE uuid = $3";

        match sqlx::query(query)
            .bind(false)
            .bind(now)
            .bind(uuid)
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error disabling 2FA: {:?}", e);
                Err(UserError::NoSuchUserFound)
            }
        }
    }
}
