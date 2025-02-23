use crate::db::data_trait::user_data_trait::UserData;
use crate::db::database::Database;
use crate::error::user_error::UserError;
use crate::models::user::{
    CheckEmailOutputResponse, CheckEmailRequest, CreateUserRequest, UpdateUserRequest,
    UpdateUserURL, User, UserResponse,
};
use actix_web::{
    get, patch, post,
    web::{Data, Json, Path},
};
use bcrypt::{hash, DEFAULT_COST};
use check_if_email_exists::{check_email, CheckEmailInput, Reachable};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn user_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_users);
    cfg.service(create_user);
    cfg.service(update_user);
    cfg.service(check_email_exists);
}

#[get("/users")]
async fn get_users(db: Data<Database>) -> Result<Json<Vec<User>>, UserError> {
    let users = Database::get_all_users(&db).await;
    match users {
        Some(users) => Ok(Json(users)),
        None => Err(UserError::NoUsersFound),
    }
}

#[post("/check-email")]
async fn check_email_exists(body: Json<CheckEmailRequest>) -> Reachable {
    let input = CheckEmailInput::new(body.email.clone());

    let result = check_email(&input).await;
    println!("{:?}", result);

    match result {
        Ok(reachable) => reachable,
        Err(e) => {
            eprintln!("Error checking email: {:?}", e);
            Reachable::Invalid
        }
    }
}

#[post("/signup")]
async fn create_user(
    body: Json<CreateUserRequest>,
    db: Data<Database>,
) -> Result<Json<UserResponse>, UserError> {
    let is_valid = body.validate();
    match is_valid {
        Ok(_) => {
            let username = body.username.clone();
            let mut buffer = Uuid::encode_buffer();
            let new_uuid = Uuid::new_v4()
                .simple()
                .encode_lower(&mut buffer)
                .to_string();
            // hash password
            let hashed_password = match hash(&body.password, DEFAULT_COST) {
                Ok(hash) => hash,
                Err(e) => {
                    eprintln!("Password hashing failed: {:?}", e);
                    return Err(UserError::UserCreationFailure);
                }
            };
            let expiration = Utc::now()
                .checked_add_signed(Duration::hours(24))
                .expect("Valid timestamp")
                .timestamp() as usize;

            let claims = Claims {
                sub: new_uuid.clone(),
                exp: expiration,
            };

            let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_key".into());
            let token = match encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret.as_ref()),
            ) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("JWT token creation failed: {:?}", e);
                    return Err(UserError::UserCreationFailure);
                }
            };

            let new_user = Database::add_user(
                &db,
                User::new(
                    new_uuid,
                    username,
                    hashed_password,
                    Utc::now().to_string(),
                    Utc::now().to_string(),
                ),
            )
            .await;

            match new_user {
                Some(user) => Ok(Json(UserResponse {
                    username: user.username,
                    access_token: token,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                })),
                None => Err(UserError::UserCreationFailure),
            }
        }
        Err(_) => Err(UserError::UserCreationFailure),
    }
}

#[patch("/users/{uuid}")]
async fn update_user(
    update_user_url: Path<UpdateUserURL>,
    body: Json<UpdateUserRequest>,
    db: Data<Database>,
) -> Result<Json<User>, UserError> {
    let is_valid = body.validate();
    match is_valid {
        Ok(_) => {
            let updated_user =
                Database::update_user(&db, update_user_url.uuid.clone(), body.username.clone())
                    .await;

            match updated_user {
                Some(user) => Ok(Json(user)),
                None => Err(UserError::NoSuchUserFound),
            }
        }
        Err(_) => Err(UserError::NoSuchUserFound),
    }
}
