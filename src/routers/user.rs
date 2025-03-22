use crate::db::data_trait::user_data_trait::UserData;
use crate::db::database::Database;
use crate::db::redis_client::RedisClient;
use crate::error::user_error::UserError;
use crate::models::user::{
    CreateUserRequest, Disable2FARequest, Enable2FARequest, Enable2FAResponse,
    GenerateBackupCodesResponse, LoginRequest, RefreshTokenRequest, TokenResponse,
    UpdateUserRequest, UpdateUserURL, UseBackupCodeForLoginRequest, User, UserResponse,
    Verify2FARequest, Verify2FAResponse,
};
use crate::services::cache_service::CacheService;
use crate::services::token_service::generate_jwt_token;
use crate::services::two_factor_service;
use actix_web::{
    patch, post,
    web::{Data, Json, Path},
};
use bcrypt::{hash, verify};
use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio;
use uuid::Uuid;
use validator::Validate;

pub fn user_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(refresh_token_endpoint)
        .service(enable_2fa)
        .service(disable_2fa)
        .service(verify_2fa)
        .service(generate_backup_codes)
        .service(login_with_backup_code);
}

const HASH_COST: u32 = 8;
const USER_CACHE_TTL: u64 = 3600;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub token_type: String,
    pub user_id: Option<String>,
}

async fn generate_token_pair(
    user_id: &str,
    redis_client: &RedisClient,
) -> Result<(String, String), UserError> {
    let token_id = Uuid::new_v4().to_string();

    let user_id_clone = user_id.to_string();

    let access_token_future =
        tokio::spawn(async move { generate_jwt_token(&user_id_clone, "access", 1, None) });

    let refresh_token_future = tokio::spawn({
        let token_id = token_id.clone();
        let user_id = user_id.to_string();
        async move { generate_jwt_token(&token_id, "refresh", 24 * 7, Some(&user_id)) }
    });

    let access_token = access_token_future
        .await
        .map_err(|_| UserError::TokenCreationFailure)??;

    let refresh_token = refresh_token_future
        .await
        .map_err(|_| UserError::TokenCreationFailure)??;

    redis_client
        .store_token_state(&token_id, user_id, 7 * 24 * 60 * 60)
        .await
        .map_err(|e| {
            eprintln!("Redis error: {:?}", e);
            UserError::TokenCreationFailure
        })?;

    Ok((access_token, refresh_token))
}

async fn validate_refresh_token(
    token: &str,
    redis_client: &RedisClient,
) -> Result<String, UserError> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_key".into());

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| UserError::InvalidRefreshToken)?;

    if token_data.claims.token_type != "refresh" {
        return Err(UserError::InvalidRefreshToken);
    }

    let token_id = token_data.claims.sub;
    let user_id = token_data
        .claims
        .user_id
        .ok_or(UserError::InvalidRefreshToken)?;

    match redis_client.validate_and_invalidate_token(&token_id).await {
        Ok(Some(stored_user_id)) => {
            if stored_user_id != user_id {
                return Err(UserError::InvalidRefreshToken);
            }
            Ok(user_id)
        }
        Ok(None) => Err(UserError::InvalidRefreshToken),
        Err(e) => {
            eprintln!("Redis error: {:?}", e);
            Err(UserError::AuthenticationFailure)
        }
    }
}

#[post("/register")]
pub async fn register(
    body: Json<CreateUserRequest>,
    db: Data<Database>,
) -> Result<Json<UserResponse>, UserError> {
    // Validate request
    body.validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    // Check if user already exists
    let existing_user_result = db.get_user_by_email(&body.email).await;
    if let Ok(_) = existing_user_result {
        return Err(UserError::UserAlreadyExists);
    }

    // Hash password with lower cost
    let hashed_password = hash(&body.password, HASH_COST).map_err(|e| {
        eprintln!("Password hashing error: {:?}", e);
        UserError::PasswordHashingFailure
    })?;

    // Create new user
    let new_uuid = Uuid::new_v4().to_string();
    let user = CreateUserRequest {
        email: body.email.clone(),
        password: hashed_password,
        name: body.name.clone(),
    };

    // Save user to database
    db.create_user(&new_uuid, &user).await?;

    // Generate token pair
    let (access_token, refresh_token_str) =
        generate_token_pair(&new_uuid, &db.redis_client).await?;

    let new_user = User::new(
        new_uuid.clone(),
        body.email.clone(),
        body.name.clone(),
        Utc::now().naive_utc(),
        Utc::now().naive_utc(),
    );

    // Cache user for future logins
    let cache_key = format!("user:email:{}", body.email);
    if let Err(e) = db
        .redis_client
        .set_cached(&cache_key, &new_user, USER_CACHE_TTL)
        .await
    {
        eprintln!("Failed to cache user: {:?}", e);
        // Continue even if caching fails
    }

    let user_response = UserResponse {
        user: new_user.into(),
        access_token,
        refresh_token: refresh_token_str,
        token_type: "Bearer".to_string(),
    };
    Ok(Json(user_response))
}

#[post("/login")]
pub async fn login(
    body: Json<LoginRequest>,
    db: Data<Database>,
) -> Result<Json<UserResponse>, UserError> {
    // Validate request
    body.validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    let cache_key = format!("user:email:{}", body.email);
    let cached_user = db.redis_client.get_cached::<User>(&cache_key).await;

    let user = match cached_user {
        Ok(Some(user)) => {
            let password_matches = verify(&body.password, &user.password).map_err(|e| {
                eprintln!("Password verification error: {:?}", e);
                UserError::AuthenticationFailure
            })?;

            if !password_matches {
                return Err(UserError::InvalidCredentials);
            }
            user
        }
        _ => {
            let user = match db.get_user_by_email(&body.email).await {
                Ok(user) => user,
                Err(UserError::NoSuchUserFound) => return Err(UserError::InvalidCredentials),
                Err(e) => return Err(e),
            };

            let password_matches = verify(&body.password, &user.password).map_err(|e| {
                eprintln!("Password verification error: {:?}", e);
                UserError::AuthenticationFailure
            })?;

            if !password_matches {
                return Err(UserError::InvalidCredentials);
            }

            if let Err(e) = db
                .redis_client
                .set_cached(&cache_key, &user, USER_CACHE_TTL)
                .await
            {
                eprintln!("Failed to cache user: {:?}", e);
                // Continue even if caching fails
            }

            user
        }
    };

    if user.two_factor_enabled {
        match &body.totp_code {
            Some(totp_code) => {
                let secret = match &user.two_factor_secret {
                    Some(secret) => secret,
                    None => return Err(UserError::TwoFactorRequired),
                };

                let is_valid = two_factor_service::verify_totp(secret, totp_code)
                    .map_err(|_| UserError::InvalidTwoFactorCode)?;

                if !is_valid {
                    return Err(UserError::InvalidTwoFactorCode);
                }
            }
            None => {
                return Err(UserError::TwoFactorRequired);
            }
        }
    }

    let (access_token, refresh_token_str) =
        generate_token_pair(&user.uuid, &db.redis_client).await?;

    let user_response = UserResponse {
        user: user.into(),
        access_token,
        refresh_token: refresh_token_str,
        token_type: "Bearer".to_string(),
    };

    Ok(Json(user_response))
}

#[post("/refresh")]
pub async fn refresh_token_endpoint(
    db: Data<Database>,
    body: Json<RefreshTokenRequest>,
) -> Result<Json<TokenResponse>, UserError> {
    let user_id = validate_refresh_token(&body.refresh_token, &db.redis_client).await?;

    let user = db.get_user_by_uuid(&user_id).await?;

    let (access_token, refresh_token_str) =
        generate_token_pair(&user.uuid, &db.redis_client).await?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token: refresh_token_str,
        token_type: "Bearer".to_string(),
    }))
}

#[patch("/users/{uuid}")]
pub async fn update_user(
    update_user_url: Path<UpdateUserURL>,
    body: Json<UpdateUserRequest>,
    db: Data<Database>,
) -> Result<Json<User>, UserError> {
    // Validate request
    body.validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    let user = db.get_user_by_uuid(&update_user_url.uuid).await?;

    let mut updated_user = user.clone();
    updated_user.email = body.email.clone();

    let result = db.update_user(&updated_user).await?;

    Ok(Json(result))
}

#[post("/users/{uuid}/enable-2fa")]
pub async fn enable_2fa(
    uuid: Path<String>,
    body: Json<Enable2FARequest>,
    db: Data<Database>,
) -> Result<Json<Enable2FAResponse>, UserError> {
    let user_id = uuid.into_inner();

    let user = db.get_user_by_uuid(&user_id).await?;

    if !verify(&body.password, &user.password).map_err(|_| UserError::AuthenticationFailure)? {
        return Err(UserError::InvalidCredentials);
    }

    if user.two_factor_enabled {
        return Err(UserError::TwoFactorAlreadyEnabled);
    }

    let secret = two_factor_service::generate_secret();

    let app_name = "Todo App";
    let totp_url = two_factor_service::generate_totp_url(&secret, &user.email, app_name);

    let qr_code = two_factor_service::generate_qr_code(&totp_url)
        .map_err(|_| UserError::QRCodeGenerationFailure)?;

    db.enable_2fa(&user_id, &secret).await?;

    let response = Enable2FAResponse {
        secret,
        qr_code,
        message: "2FA đã được thiết lập. Vui lòng quét mã QR và xác minh mã để hoàn tất."
            .to_string(),
    };

    Ok(Json(response))
}

#[post("/users/{uuid}/disable-2fa")]
pub async fn disable_2fa(
    uuid: Path<String>,
    body: Json<Disable2FARequest>,
    db: Data<Database>,
) -> Result<Json<Verify2FAResponse>, UserError> {
    let user_id = uuid.into_inner();

    let user = db.get_user_by_uuid(&user_id).await?;

    if !verify(&body.password, &user.password).map_err(|_| UserError::AuthenticationFailure)? {
        return Err(UserError::InvalidCredentials);
    }

    if !user.two_factor_enabled {
        return Err(UserError::TwoFactorNotEnabled);
    }

    let secret = match &user.two_factor_secret {
        Some(secret) => secret,
        None => return Err(UserError::TwoFactorNotEnabled),
    };

    let is_valid = two_factor_service::verify_totp(secret, &body.code)
        .map_err(|_| UserError::InvalidTwoFactorCode)?;

    if !is_valid {
        return Err(UserError::InvalidTwoFactorCode);
    }

    db.disable_2fa(&user_id).await?;

    let response = Verify2FAResponse {
        success: true,
        message: "2FA đã được tắt thành công.".to_string(),
    };

    Ok(Json(response))
}

#[post("/users/{uuid}/verify-2fa")]
pub async fn verify_2fa(
    uuid: Path<String>,
    body: Json<Verify2FARequest>,
    db: Data<Database>,
) -> Result<Json<Verify2FAResponse>, UserError> {
    let user_id = uuid.into_inner();

    let user = db.get_user_by_uuid(&user_id).await?;

    let secret = match &user.two_factor_secret {
        Some(secret) => secret,
        None => return Err(UserError::TwoFactorNotEnabled),
    };

    let is_valid = two_factor_service::verify_totp(secret, &body.code)
        .map_err(|_| UserError::InvalidTwoFactorCode)?;

    if !is_valid {
        return Err(UserError::InvalidTwoFactorCode);
    }

    db.verify_2fa(&user_id).await?;

    let response = Verify2FAResponse {
        success: true,
        message: "2FA đã được xác minh và kích hoạt thành công.".to_string(),
    };

    Ok(Json(response))
}

#[post("/users/{uuid}/2fa/backup-codes")]
pub async fn generate_backup_codes(
    uuid: Path<String>,
    body: Json<Verify2FARequest>,
    db: Data<Database>,
) -> Result<Json<GenerateBackupCodesResponse>, UserError> {
    let user = db.get_user_by_uuid(&uuid).await?;

    if !user.two_factor_enabled {
        return Err(UserError::BadRequest(
            "2FA is not enabled for this user".to_string(),
        ));
    }

    if user.two_factor_secret.is_none() {
        return Err(UserError::BadRequest("2FA secret not found".to_string()));
    }

    let secret = user.two_factor_secret.as_ref().unwrap();
    let is_valid = match two_factor_service::verify_totp(secret, &body.code) {
        Ok(valid) => valid,
        Err(e) => {
            return Err(UserError::BadRequest(format!(
                "Error verifying TOTP: {}",
                e
            )))
        }
    };

    if !is_valid {
        return Err(UserError::BadRequest("Invalid 2FA code".to_string()));
    }

    // Invalidate previous backup codes
    let mut updated_user = user.clone();
    updated_user.backup_codes = None;
    db.update_user(&updated_user).await?;

    // Generate new backup codes
    let (plain_codes, hashed_codes) = two_factor_service::generate_backup_codes(None);

    let formatted_codes: Vec<String> = plain_codes
        .iter()
        .map(|code| two_factor_service::format_backup_code(code))
        .collect();

    updated_user.backup_codes = Some(hashed_codes);
    db.update_user(&updated_user).await?;

    Ok(Json(GenerateBackupCodesResponse {
        backup_codes: formatted_codes,
        message: "Backup codes generated successfully".to_string(),
    }))
}

#[post("/login/backup")]
pub async fn login_with_backup_code(
    body: Json<UseBackupCodeForLoginRequest>,
    db: Data<Database>,
) -> Result<Json<UserResponse>, UserError> {
    let user = db.get_user_by_email(&body.email).await?;

    let is_valid = verify(&body.password, &user.password)
        .map_err(|_| UserError::BadRequest("Invalid email or password".to_string()))?;

    if !is_valid {
        return Err(UserError::BadRequest(
            "Invalid email or password".to_string(),
        ));
    }

    if !user.two_factor_enabled {
        return Err(UserError::BadRequest(
            "2FA is not enabled for this user".to_string(),
        ));
    }

    if user.backup_codes.is_none() || user.backup_codes.as_ref().unwrap().is_empty() {
        return Err(UserError::BadRequest(
            "No backup codes available".to_string(),
        ));
    }

    let backup_code = body.backup_code.replace("-", ""); // Loại bỏ dấu gạch ngang nếu có
    let backup_codes = user.backup_codes.as_ref().unwrap();
    let code_index = two_factor_service::verify_backup_code(&backup_code, backup_codes);

    if let Some(index) = code_index {
        let mut updated_user = user.clone();
        let mut updated_codes = backup_codes.clone();
        updated_codes.remove(index);
        updated_user.backup_codes = Some(updated_codes);
        db.update_user(&updated_user).await?;

        let (access_token, refresh_token) =
            generate_token_pair(&user.uuid, &db.redis_client).await?;

        Ok(Json(UserResponse {
            user: user.into(),
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
        }))
    } else {
        Err(UserError::BadRequest("Invalid backup code".to_string()))
    }
}
