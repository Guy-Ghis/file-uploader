use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{env, fs};
use tokio::io::AsyncWriteExt;

use crate::auth::validate_token;
use crate::metadata::{create_upload_response, log_upload_metadata};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
    pub timestamp: String,
}

/// Health check endpoint
pub async fn health_check() -> ActixResult<HttpResponse> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        message: "Upload proxy service is running".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };
    Ok(HttpResponse::Ok().json(response))
}

/// File upload handler - implements the complete assignment flow
pub async fn upload_file(
    mut payload: Multipart,
    auth: BearerAuth,
    _req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    log::info!("=== UPLOAD HANDLER CALLED ===");
    log::info!("Starting file upload process");
    println!("=== BACKEND: Upload handler was called! ===");

    // Step 1: Authorization Check - Validate JWT and get user
    log::info!("Step 1: Validating JWT token");
    let user = match validate_token(auth.token()).await {
        Ok(user) => {
            log::info!("Token validation successful for user: {}", user);
            user
        }
        Err(e) => {
            log::error!("Token validation failed: {:?}", e);
            return Err(e);
        }
    };

    // Step 2: File Processing - Prepare upload directory
    log::info!("Step 2: Preparing file storage");
    let uploads_path = env::var("UPLOADS_DIR").unwrap_or_else(|_| "./uploads".to_string());
    let uploads_dir = Path::new(&uploads_path);
    if !uploads_dir.exists() {
        fs::create_dir_all(uploads_dir).map_err(|e| {
            log::error!("Failed to create uploads directory: {}", e);
            actix_web::error::ErrorInternalServerError(format!(
                "Failed to create uploads directory: {}",
                e
            ))
        })?;
    }

    let mut filename = String::new();
    let mut total_bytes = 0u64;

    // Step 3: Stream multipart upload and write directly to disk
    log::info!("Step 3: Processing multipart upload stream");
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            log::error!("Failed to read multipart field: {}", e);
            actix_web::error::ErrorBadRequest(format!("Invalid multipart data: {}", e))
        })?;

        // Extract filename from Content-Disposition header
        filename = field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
            .map(|f| f.to_string())
            .unwrap_or_else(|| format!("file_{}", Utc::now().timestamp()));

        log::info!("Processing file: {}", filename);
        let filepath = uploads_dir.join(&filename);

        // Create file and stream data directly to disk
        let mut file = tokio::fs::File::create(&filepath).await.map_err(|e| {
            log::error!("Failed to create file {}: {}", filepath.display(), e);
            actix_web::error::ErrorInternalServerError(format!("Failed to create file: {}", e))
        })?;

        // Stream file chunks directly to disk
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| {
                log::error!("Failed to read chunk: {}", e);
                actix_web::error::ErrorBadRequest(format!("Failed to read file data: {}", e))
            })?;

            total_bytes += data.len() as u64;
            file.write_all(&data).await.map_err(|e| {
                log::error!("Failed to write chunk to file: {}", e);
                actix_web::error::ErrorInternalServerError(format!("Failed to write file: {}", e))
            })?;
        }

        // Ensure data is written to disk
        file.flush().await.map_err(|e| {
            log::error!("Failed to flush file: {}", e);
            actix_web::error::ErrorInternalServerError(format!("Failed to flush file: {}", e))
        })?;
    }

    if filename.is_empty() {
        log::error!("No file was uploaded");
        return Err(actix_web::error::ErrorBadRequest("No file uploaded"));
    }

    log::info!(
        "File upload completed: {} ({} bytes)",
        filename,
        total_bytes
    );

    // Step 4: Metadata Logging - Create and append metadata entry
    log::info!("Step 4: Logging upload metadata");
    let metadata_file = env::var("METADATA_FILE").unwrap_or_else(|_| "./uploads.json".to_string());
    log_upload_metadata(filename.clone(), user.clone(), total_bytes, &metadata_file)?;

    log::info!(
        "Upload process completed successfully for file: {}",
        filename
    );

    // Return success response with file details
    let response = create_upload_response(filename, user, total_bytes);
    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize)]
pub struct TokenExchangeRequest {
    pub code: String,
    pub code_verifier: String,
    pub redirect_uri: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
}

/// Token exchange endpoint - proxies token request to Keycloak
pub async fn exchange_token(
    token_request: web::Json<TokenExchangeRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    log::info!("Processing token exchange request");

    let keycloak_url = env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set");
    let keycloak_realm = env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "upload-realm".to_string());
    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");

    let token_url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        keycloak_url, keycloak_realm
    );

    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("code", &token_request.code),
        ("redirect_uri", &token_request.redirect_uri),
        ("code_verifier", &token_request.code_verifier),
    ];

    match client.post(&token_url).form(&params).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(token_data) => {
                        log::info!("Token exchange successful");
                        Ok(HttpResponse::Ok().json(token_data))
                    }
                    Err(e) => {
                        log::error!("Failed to parse token response: {}", e);
                        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Failed to parse token response"
                        })))
                    }
                }
            } else {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                log::error!("Token exchange failed: {}", error_text);
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Token exchange failed",
                    "details": error_text
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to connect to Keycloak: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to connect to Keycloak"
            })))
        }
    }
}
