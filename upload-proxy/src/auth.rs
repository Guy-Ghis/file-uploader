use actix_web;
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web::dev::ServiceRequest;
use jsonwebtoken::{decode, DecodingKey, Validation, errors::ErrorKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Audience {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: Option<String>, // Now optional to avoid hard failure
    pub exp: usize,
    #[serde(default)]
    pub aud: Option<Audience>,
}

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

pub async fn validate_token(token: &str) -> Result<String, actix_web::Error> {
    log::info!("=== JWT VALIDATION START ===");
    log::info!("Token length: {}", token.len());
    log::info!("Token preview: {}...", &token[..token.len().min(50)]);

    let keycloak_url = env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set");
    let keycloak_realm = env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "upload-realm".to_string());
    let _client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set");
    let _client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");
    let jwt_audience =
        env::var("JWT_AUDIENCE").unwrap_or_else(|_| "account,upload-client".to_string());

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
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to fetch JWKS: {}", e)))?
        .json()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to parse JWKS: {}", e)))?;

    let token_header = jsonwebtoken::decode_header(token)
        .map_err(|e| actix_web::error::ErrorUnauthorized(format!("Invalid token header: {}", e)))?;

    let kid = token_header.kid.ok_or_else(|| actix_web::error::ErrorUnauthorized("Token missing key ID"))?;

    let keys_array = jwks["keys"].as_array().ok_or_else(|| actix_web::error::ErrorUnauthorized("Invalid JWKS format"))?;
    let matching_key = keys_array.iter().find(|key| key["kid"].as_str() == Some(&kid))
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("No matching key found"))?;

    let jwk_n = matching_key["n"].as_str().ok_or_else(|| actix_web::error::ErrorUnauthorized("Invalid JWK"))?;
    let jwk_e = matching_key["e"].as_str().unwrap_or("AQAB");

    let decoding_key = DecodingKey::from_rsa_components(jwk_n, jwk_e)
        .map_err(|e| actix_web::error::ErrorUnauthorized(format!("Failed to create decoding key: {}", e)))?;

    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    let audiences: Vec<&str> = jwt_audience.split(',').map(|s| s.trim()).collect();
    validation.set_audience(&audiences);
    validation.set_issuer(&[&format!("{}/realms/{}", keycloak_url, keycloak_realm)]);
    validation.leeway = 60;

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            log::info!("Token validated successfully!");
            let sub = token_data.claims.sub.unwrap_or_else(|| "unknown".to_string());
            Ok(sub)
        }
        Err(err) => {
            match err.kind() {
                ErrorKind::ExpiredSignature => {
                    log::warn!("Token expired — session timeout.");
                    Err(actix_web::error::ErrorUnauthorized("Session expired, please log in again"))
                }
                _ => {
                    log::error!("JWT validation failed: {}", err);
                    Err(actix_web::error::ErrorUnauthorized(format!("Invalid token: {}", err)))
                }
            }
        }
    }
}
