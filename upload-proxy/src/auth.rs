use actix_web;
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web::dev::ServiceRequest;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

// Custom deserialization for aud claim (string or array)
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
// #[derive(Debug)]
#[derive(Debug)]
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

/// Authentication validator for middleware
pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let token = credentials.token();
    log::info!("=== AUTHENTICATION MIDDLEWARE ===");
    log::info!("Validating token in middleware");
    
    match validate_token(token).await {
        Ok(user) => {
            log::info!("Authentication successful for user: {}", user);
            Ok(req)
        }
        Err(e) => {
            log::error!("Authentication failed: {:?}", e);
            let config = req.app_data::<Config>().cloned().unwrap_or_default();
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

/// Validates JWT token against Keycloak and returns user ID
pub async fn validate_token(token: &str) -> Result<String, actix_web::Error> {
    log::info!("=== JWT VALIDATION START ===");
    log::info!("Token length: {}", token.len());
    log::info!("Token preview: {}...", &token[..token.len().min(50)]);
    
    let keycloak_url = env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set");
    let keycloak_realm = env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "upload-realm".to_string());
    // Note: CLIENT_ID and CLIENT_SECRET are loaded for completeness but not used in JWT validation
    let _client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set");
    let _client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");
    let jwt_audience =
        env::var("JWT_AUDIENCE").unwrap_or_else(|_| "account,upload-client".to_string());

    log::info!("Keycloak URL: {}", keycloak_url);
    log::info!("Keycloak Realm: {}", keycloak_realm);
    log::info!("JWT Audience: {}", jwt_audience);

    // Fetch Keycloak's public key
    let jwks_url = format!(
        "{}/realms/{}/protocol/openid-connect/certs",
        keycloak_url, keycloak_realm
    );
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

    log::info!("JWKS fetched successfully");
    log::info!("JWKS keys count: {}", jwks["keys"].as_array().map(|a| a.len()).unwrap_or(0));

    // Find the correct key by matching the 'kid' (key ID) from the token
    let token_header = jsonwebtoken::decode_header(token).map_err(|e| {
        log::error!("Failed to decode token header: {}", e);
        actix_web::error::ErrorUnauthorized(format!("Invalid token header: {}", e))
    })?;

    let kid = token_header.kid.ok_or_else(|| {
        log::error!("Token missing 'kid' field");
        actix_web::error::ErrorUnauthorized("Token missing key ID")
    })?;

    log::info!("Token key ID (kid): {}", kid);

    // Find the matching key in JWKS
    let keys_array = jwks["keys"].as_array().ok_or_else(|| {
        log::error!("Invalid JWKS format: keys is not an array");
        actix_web::error::ErrorUnauthorized("Invalid JWKS format")
    })?;

    let matching_key = keys_array.iter().find(|key| {
        key["kid"].as_str() == Some(&kid)
    }).ok_or_else(|| {
        log::error!("No matching key found for kid: {}", kid);
        actix_web::error::ErrorUnauthorized("No matching key found")
    })?;

    log::info!("Found matching key for kid: {}", kid);

    let jwk_n = matching_key["n"].as_str().ok_or_else(|| {
        log::error!("Invalid JWK: missing 'n' field");
        actix_web::error::ErrorUnauthorized("Invalid JWK")
    })?;

    let jwk_e = matching_key["e"].as_str().unwrap_or("AQAB");
    log::info!("Using JWK exponent: {}", jwk_e);

    // Decode and validate JWT
    let decoding_key = DecodingKey::from_rsa_components(jwk_n, jwk_e).map_err(|e| {
        log::error!("Failed to create decoding key: {}", e);
        actix_web::error::ErrorUnauthorized(format!("Failed to create decoding key: {}", e))
    })?;

    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    let audiences: Vec<&str> = jwt_audience.split(',').map(|s| s.trim()).collect();
    validation.set_audience(&audiences);
    validation.set_issuer(&[&format!("{}/realms/{}", keycloak_url, keycloak_realm)]);
    validation.leeway = 60; // allow 60s of clock skew

    log::info!("Attempting to decode token with audiences: {:?}", audiences);
    log::info!("Expected issuer: {}/realms/{}", keycloak_url, keycloak_realm);
    
    let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
        log::error!("JWT validation failed: {}", e);
        log::error!("Token: {}", token);
        actix_web::error::ErrorUnauthorized(format!("Invalid token: {}", e))
    })?;

    log::info!("Token validated successfully!");
    log::info!("Token claims - sub: {}, exp: {}", token_data.claims.sub, token_data.claims.exp);
    log::info!("Token audience: {:?}", token_data.claims.aud);
    log::info!("=== JWT VALIDATION END ===");
    
    Ok(token_data.claims.sub)
}
