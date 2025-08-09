use actix_web::{middleware, web, App, HttpServer};
use actix_cors::Cors;

mod auth;
mod handlers;
mod metadata;

use handlers::{health_check, upload_file};



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    log::info!("Starting server on 0.0.0.0:3000");
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(Cors::default()
                .allowed_origin("http://10.153.115.29:8000")
                .allowed_origin("http://localhost:8000")
                .allowed_origin("http://127.0.0.1:8000")
                .allowed_origin("file://")
                .allow_any_origin()
                .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                .allowed_headers(vec!["Authorization", "Content-Type", "Accept"])
                .supports_credentials()
                .max_age(3600))
            .route("/health", web::get().to(health_check))
            .route("/upload", web::post().to(upload_file))
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}