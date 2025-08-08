use actix_multipart::Multipart;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_cors::Cors;
use chrono::Utc;
use futures::StreamExt;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::env;
use std::fs::{self, OpenOptions};
use std::path::Path;
use tokio::io::AsyncWriteExt;

// Custom deserialization for aud claim (string or array)
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Audience {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    #[serde(default)]
    aud: Option<Audience>, // Handle string or array
}

#[derive(Serialize, Deserialize)]
struct UploadMetadata {
    filename: String,
    user: String,
    timestamp: String,
}

async fn validate_token(token: &str) -> Result<String, actix_web::Error> {
    let keycloak_url = env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set");
    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");

    // Fetch Keycloak's public key
    let jwks_url = format!("{}/realms/upload-realm/protocol/openid-connect/certs", keycloak_url);
    log::info!("Fetching JWKS from: {}", jwks_url);
    let client = reqwest::Client::new();
    let jwks: Value = client
        .get(&jwks_url)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to fetch JWKS: {}", e);
            actix_web::error::ErrorInternalServerError(format!("Failed to fetch JWKS: {}", e))
        })?
        .json()
        .await
        .map_err(|e| {
            log::error!("Failed to parse JWKS: {}", e);
            actix_web::error::ErrorInternalServerError(format!("Failed to parse JWKS: {}", e))
        })?;
    let jwk = jwks["keys"][0]["n"]
        .as_str()
        .ok_or_else(|| {
            log::error!("Invalid JWK: missing 'n' field");
            actix_web::error::ErrorUnauthorized("Invalid JWK")
        })?;

    // Decode and validate JWT
    let decoding_key = DecodingKey::from_rsa_components(jwk, "AQAB")
        .map_err(|e| {
            log::error!("Failed to create decoding key: {}", e);
            actix_web::error::ErrorUnauthorized(format!("Failed to create decoding key: {}", e))
        })?;
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&["account", "upload-client"]);
    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| {
            log::error!("JWT validation failed: {}", e);
            actix_web::error::ErrorUnauthorized(format!("Invalid token: {}", e))
        })?;

    log::info!("Token validated for user: {}", token_data.claims.sub);
    Ok(token_data.claims.sub)
}

async fn upload_file(
    mut payload: Multipart,
    auth: BearerAuth,
    _req: HttpRequest,
) -> impl Responder {
    // Validate JWT and get user
    let user = match validate_token(auth.token()).await {
        Ok(user) => user,
        Err(e) => return Err(e),
    };

    let uploads_dir = Path::new("./uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(uploads_dir)?;
    }

    let mut filename = String::new();
    while let Some(item) = payload.next().await {
        let mut field = item?;
        filename = field
            .content_disposition()
            .expect("Missing Content-Disposition")
            .get_filename()
            .map(|f| f.to_string())
            .unwrap_or_else(|| format!("file_{}", Utc::now().timestamp()));
        let filepath = uploads_dir.join(&filename);

        let mut file = tokio::fs::File::create(&filepath).await?;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            file.write_all(&data).await?;
        }
    }

    // Log metadata to uploads.json
    let metadata = UploadMetadata {
        filename,
        user,
        timestamp: Utc::now().to_rfc3339(),
    };

    let mut uploads = if Path::new("./uploads.json").exists() {
        let content = fs::read_to_string("./uploads.json")?;
        serde_json::from_str::<Vec<UploadMetadata>>(&content).unwrap_or_default()
    } else {
        vec![]
    };

    uploads.push(metadata);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("./uploads.json")?;
    serde_json::to_writer(file, &uploads)?;

    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}

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
                .allowed_methods(vec!["POST"])
                .allowed_headers(vec!["Authorization", "Content-Type"])
                .max_age(3600))
            .route("/upload", web::post().to(upload_file))
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}