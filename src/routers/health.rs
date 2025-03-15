use actix_web::{get, web, HttpResponse};
use crate::db::database::Database;
use crate::swagger::ApiResponseHealthResponse;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    #[schema(example = "ok")]
    pub status: String,
    #[schema(example = "1.0.0")]
    pub version: String,
    #[schema(example = "1633027200")]
    pub timestamp: u64,
    #[schema(example = "connected")]
    pub database: String,
}

pub fn health_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
}

/// Kiểm tra trạng thái hoạt động của API
///
/// Endpoint này trả về thông tin về trạng thái hoạt động của API, bao gồm:
/// - Trạng thái chung của API
/// - Phiên bản hiện tại
/// - Thời gian hiện tại
/// - Trạng thái kết nối cơ sở dữ liệu
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "Trạng thái hoạt động của API", body = ApiResponseHealthResponse)
    )
)]
#[get("/health")]
async fn health(db: web::Data<Database>) -> HttpResponse {
    // Kiểm tra kết nối cơ sở dữ liệu
    let db_status = match sqlx::query("SELECT 1").fetch_one(&db.pool).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };
    
    // Lấy thời gian hiện tại
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Lấy phiên bản từ Cargo.toml
    let version = env!("CARGO_PKG_VERSION", "0.1.0");
    
    let health_data = HealthResponse {
        status: "ok".to_string(),
        version: version.to_string(),
        timestamp,
        database: db_status.to_string(),
    };
    
    let response = ApiResponseHealthResponse {
        success: true,
        message: "Health check successful".to_string(),
        data: Some(health_data),
    };
    
    HttpResponse::Ok().json(response)
} 