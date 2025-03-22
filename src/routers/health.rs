use crate::db::database::Database;
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::app::ApiResponseHealthResponse;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: u64,
    pub database: String,
    pub redis: String,
}

pub fn health_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
}


#[get("/health")]
async fn health(db: web::Data<Database>) -> HttpResponse {
    let db_status = match sqlx::query("SELECT 1").fetch_one(&db.pool).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let redis_status = match db.redis_client.check_connection().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let version = env!("CARGO_PKG_VERSION", "0.1.0");

    let health_data = HealthResponse {
        status: "ok".to_string(),
        version: version.to_string(),
        timestamp,
        database: db_status.to_string(),
        redis: redis_status.to_string(),
    };

    let response = ApiResponseHealthResponse {
        success: true,
        message: "Health check successful".to_string(),
        data: Some(health_data),
    };

    HttpResponse::Ok().json(response)
}
