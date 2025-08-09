use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::Utc;
use futures::StreamExt;
use serde::Serialize;
use std::fs;
use std::path::Path;
use tokio::io::AsyncWriteExt;

use crate::auth::validate_token;
use crate::metadata::{log_upload_metadata, create_upload_response};

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
    log::info!("Starting file upload process");
    
    // Step 1: Authorization Check - Validate JWT and get user
    log::info!("Step 1: Validating JWT token");
    let user = match validate_token(auth.token()).await {
        Ok(user) => {
            log::info!("Token validation successful for user: {}", user);
            user
        },
        Err(e) => {
            log::error!("Token validation failed: {:?}", e);
            return Err(e);
        }
    };

    // Step 2: File Processing - Prepare upload directory
    log::info!("Step 2: Preparing file storage");
    let uploads_dir = Path::new("./uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(uploads_dir).map_err(|e| {
            log::error!("Failed to create uploads directory: {}", e);
            actix_web::error::ErrorInternalServerError(format!("Failed to create uploads directory: {}", e))
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

    log::info!("File upload completed: {} ({} bytes)", filename, total_bytes);

    // Step 4: Metadata Logging - Create and append metadata entry
    log::info!("Step 4: Logging upload metadata");
    log_upload_metadata(filename.clone(), user.clone(), total_bytes)?;

    log::info!("Upload process completed successfully for file: {}", filename);
    
    // Return success response with file details
    let response = create_upload_response(filename, user, total_bytes);
    Ok(HttpResponse::Ok().json(response))
}
