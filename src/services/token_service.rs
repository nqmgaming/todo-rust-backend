use crate::error::user_error::UserError;
use crate::routers::user::Claims;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};

/// Tạo JWT token
pub fn generate_jwt_token(
    subject: &str,
    token_type: &str,
    expires_in_hours: i64,
    user_id: Option<&str>,
) -> Result<String, UserError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expires_in_hours))
        .expect("Valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: subject.to_string(),
        exp: expiration,
        token_type: token_type.to_string(),
        user_id: user_id.map(|id| id.to_string()),
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_key".into());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| {
        eprintln!("JWT token creation failed: {:?}", e);
        UserError::AuthenticationFailure
    })
}
