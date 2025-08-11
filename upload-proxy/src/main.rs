use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;

mod auth;
mod handlers;
mod metadata;

use auth::validator;
use handlers::{exchange_token, health_check, upload_file, refresh_token};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    env_logger::init();

    let backend_port = std::env::var("BACKEND_PORT").unwrap_or_else(|_| "3000".to_string());
    log::info!("Starting server on 0.0.0.0:{}", backend_port);
    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:8000,http://127.0.0.1:8000".to_string());
    let origins: Vec<String> = allowed_origins
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        log::info!("CORS configured for origins: {:?}", origins);

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .route("/health", web::get().to(health_check))
            .route("/token", web::post().to(exchange_token))
            .route("/refresh", web::post().to(refresh_token))
            .service(
                web::scope("/api")
                    .wrap(HttpAuthentication::bearer(validator))
                    .route("/upload", web::post().to(upload_file))
            )
    })
    .bind(format!("0.0.0.0:{}", backend_port))?
    .run()
    .await
}
