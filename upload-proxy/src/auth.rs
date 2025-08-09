use actix_web;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

// Custom deserialization for aud claim (string or array)
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Audience {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    #[serde(default)]
    pub aud: Option<Audience>, // Handle string or array
}

/// Validates JWT token against Keycloak and returns user ID
pub async fn validate_token(token: &str) -> Result<String, actix_web::Error> {
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
