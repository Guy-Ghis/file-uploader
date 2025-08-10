use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct UploadMetadata {
    pub filename: String,
    pub user: String,
    pub timestamp: String,
    pub size_bytes: u64,
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub status: String,
    pub message: String,
    pub filename: String,
    pub user: String,
    pub size_bytes: u64,
    pub timestamp: String,
}

/// Logs upload metadata to uploads.json file
pub fn log_upload_metadata(
    filename: String,
    user: String,
    size_bytes: u64,
    metadata_file_path: &str,
) -> Result<(), actix_web::Error> {
    log::info!("Logging upload metadata for file: {}", filename);

    let metadata = UploadMetadata {
        filename: filename.clone(),
        user: user.clone(),
        timestamp: Utc::now().to_rfc3339(),
        size_bytes,
    };

    // Read existing metadata or create new vector
    let mut uploads = if Path::new(metadata_file_path).exists() {
        let content = fs::read_to_string(metadata_file_path).map_err(|e| {
            log::error!("Failed to read {}: {}", metadata_file_path, e);
            actix_web::error::ErrorInternalServerError(format!("Failed to read metadata: {}", e))
        })?;
        serde_json::from_str::<Vec<UploadMetadata>>(&content).unwrap_or_else(|e| {
            log::warn!(
                "Failed to parse {}, creating new: {}",
                metadata_file_path,
                e
            );
            vec![]
        })
    } else {
        vec![]
    };

    // Append new metadata entry
    uploads.push(metadata);

    // Write updated metadata back to file
    let metadata_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(metadata_file_path)
        .map_err(|e| {
            log::error!("Failed to open {} for writing: {}", metadata_file_path, e);
            actix_web::error::ErrorInternalServerError(format!(
                "Failed to open metadata file: {}",
                e
            ))
        })?;

    serde_json::to_writer_pretty(metadata_file, &uploads).map_err(|e| {
        log::error!("Failed to write metadata: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Failed to write metadata: {}", e))
    })?;

    log::info!("Successfully logged metadata for file: {}", filename);
    Ok(())
}

/// Creates a successful upload response
pub fn create_upload_response(filename: String, user: String, size_bytes: u64) -> UploadResponse {
    UploadResponse {
        status: "success".to_string(),
        message: "File uploaded successfully".to_string(),
        filename,
        user,
        size_bytes,
        timestamp: Utc::now().to_rfc3339(),
    }
}
