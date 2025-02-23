use crate::db::database::Database;
use crate::models::user::{CreateUserRequest, UpdateUserRequest, User};
use async_trait::async_trait;

#[async_trait]
pub trait UserData {
    async fn add_user(&self, user: User) -> Option<User>;
    async fn get_all_users(&self) -> Option<Vec<User>>;
    async fn update_user(&self, uuid: String, username: String) -> Option<User>;
}

#[async_trait]
impl UserData for Database {
    async fn add_user(&self, user: User) -> Option<User> {
        let existing_user: Option<User> = self
            .client
            .select(("user", user.username.clone()))
            .await
            .ok()?;
        if existing_user.is_some() {
            println!("User already exists");
            return None;
        }

        let result = self
            .client
            .create(("user", user.uuid.clone()))
            .content(user)
            .await;
        match result {
            Ok(Some(user)) => Some(user),
            Ok(None) => {
                eprintln!("No user was added (got Ok(None))");
                None
            }
            Err(e) => {
                eprintln!("Error adding user: {:?}", e);
                None
            }
        }
    }

    async fn get_all_users(&self) -> Option<Vec<User>> {
        todo!()
    }

    async fn update_user(&self, uuid: String, username: String) -> Option<User> {
        todo!()
    }
}
