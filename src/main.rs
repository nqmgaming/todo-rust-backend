mod db;
mod error;
mod middleware;
mod models;
mod routers;
mod services;

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use db::database::Database;
use dotenv::dotenv;
use env_logger::Env;
use log::{info, warn};
use middleware::auth::{validator, TodoOwnershipChecker};
use routers::{health::health_routes, todo::todo_routes, user::user_routes};

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
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            "postgres", "postgres", "localhost", "5432", "postgres"
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
    let database = Database::init().await;
    info!("Database connection established successfully");

    let db_data = Data::new(database);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_origin("https://todo.quickmem.app")
            .allow_any_method()
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .expose_headers(vec![header::AUTHORIZATION])
            .supports_credentials()
            .max_age(3600);

        let auth = HttpAuthentication::bearer(validator);
        let _todo_ownership_checker = TodoOwnershipChecker::new(db_data.clone());

        App::new()
            .wrap(cors)
            .wrap(Logger::new("%a %r %s %b %{Referer}i %{User-Agent}i %T"))
            .app_data(db_data.clone())
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
