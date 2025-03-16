mod db;
mod error;
mod middleware;
mod models;
mod routers;
mod swagger;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use db::database::Database;
use dotenv::dotenv;
use env_logger::Env;
use log::{error, info, warn};
use middleware::auth::{validator, TodoOwnershipChecker};
use routers::{health::health_routes, todo::todo_routes, user::user_routes};
use swagger::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Cấu hình logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .format_module_path(true)
        .init();

    info!("Starting application...");

    // Set up DATABASE_URL environment variable
    if std::env::var("DATABASE_URL").is_err() {
        let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| {
            warn!("DB_HOST not set, using default");
            "localhost".to_string()
        });
        let db_port = std::env::var("DB_PORT").unwrap_or_else(|_| {
            warn!("DB_PORT not set, using default");
            "5432".to_string()
        });
        let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| {
            warn!("DB_NAME not set, using default");
            "postgres".to_string()
        });
        let db_user = std::env::var("DB_USER").unwrap_or_else(|_| {
            warn!("DB_USER not set, using default");
            "postgres".to_string()
        });
        let db_password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| {
            warn!("DB_PASSWORD not set, using default");
            "postgres".to_string()
        });

        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );
        std::env::set_var("DATABASE_URL", database_url);
        info!("DATABASE_URL set from environment variables");
    }

    // Set JWT_SECRET if not set
    if std::env::var("JWT_SECRET").is_err() {
        warn!("JWT_SECRET not set, using default (insecure)");
        std::env::set_var("JWT_SECRET", "todoapp");
    }

    // Set Redis URL if not set
    if std::env::var("REDIS_URL").is_err() {
        warn!("REDIS_URL not set, using default");
        std::env::set_var("REDIS_URL", "redis://127.0.0.1:6379");
    }

    info!("Initializing database connection...");
    let database = match Database::init().await {
        Ok(db) => {
            info!("Database connection established successfully");
            db
        }
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            panic!("Database initialization failed");
        }
    };

    let db_data = Data::new(database);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        let auth = HttpAuthentication::bearer(validator);
        let _todo_ownership_checker = TodoOwnershipChecker::new(db_data.clone());

        App::new()
            .wrap(cors)
            .wrap(Logger::new("%a %r %s %b %{Referer}i %{User-Agent}i %T"))
            .app_data(db_data.clone())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .service(
                actix_web::web::scope("/api")
                    .configure(health_routes)
                    .service(
                        actix_web::web::scope("/v1").configure(user_routes).service(
                            actix_web::web::scope("/todos")
                                .wrap(auth)
                                .configure(todo_routes),
                        ),
                    ),
            )
    })
    .bind(("0.0.0.0", 8080))?;

    info!("Server started at http://127.0.0.1:8080");
    info!("Swagger UI available at http://127.0.0.1:8080/swagger-ui/");
    server.run().await
}
