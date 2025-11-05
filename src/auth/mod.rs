use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,          // User ID
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

#[derive(Clone)]
pub struct AuthState {
    pub keycloak_url: String,
    pub realm: String,
    pub jwks_cache: Arc<RwLock<JwksCache>>,
}

pub struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    last_update: std::time::Instant,
    ttl: std::time::Duration,
}

impl JwksCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            keys: HashMap::new(),
            last_update: std::time::Instant::now(),
            ttl: std::time::Duration::from_secs(ttl_secs),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.last_update.elapsed() > self.ttl
    }
}

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Deserialize)]
struct JwkKey {
    kid: String,
    #[serde(rename = "use")]
    key_use: Option<String>,
    n: String,
    e: String,
}

impl AuthState {
    pub fn new(keycloak_url: String, realm: String, cache_ttl: u64) -> Self {
        Self {
            keycloak_url,
            realm,
            jwks_cache: Arc::new(RwLock::new(JwksCache::new(cache_ttl))),
        }
    }

    async fn fetch_jwks(&self) -> Result<HashMap<String, DecodingKey>, String> {
        let url = format!(
            "{}/protocol/openid-connect/certs",
            self.keycloak_url
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;

        let jwks: JwksResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JWKS: {}", e))?;

        let mut keys = HashMap::new();
        for key in jwks.keys {
            if key.key_use.as_deref() == Some("sig") || key.key_use.is_none() {
                match DecodingKey::from_rsa_components(&key.n, &key.e) {
                    Ok(decoding_key) => {
                        keys.insert(key.kid, decoding_key);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create decoding key: {}", e);
                    }
                }
            }
        }

        Ok(keys)
    }

    pub async fn get_decoding_key(&self, kid: &str) -> Result<DecodingKey, String> {
        // Check if cache is expired
        {
            let cache = self.jwks_cache.read().await;
            if !cache.is_expired() {
                if let Some(key) = cache.keys.get(kid) {
                    return Ok(key.clone());
                }
            }
        }

        // Refresh cache
        let keys = self.fetch_jwks().await?;
        let key = keys
            .get(kid)
            .ok_or_else(|| format!("Key with kid '{}' not found", kid))?
            .clone();

        // Update cache
        {
            let mut cache = self.jwks_cache.write().await;
            cache.keys = keys;
            cache.last_update = std::time::Instant::now();
        }

        Ok(key)
    }

    pub async fn validate_token(&self, token: &str) -> Result<Claims, String> {
        let header = decode_header(token)
            .map_err(|e| format!("Invalid token header: {}", e))?;

        let kid = header
            .kid
            .ok_or_else(|| "Token header missing 'kid'".to_string())?;

        let key = self.get_decoding_key(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        // Allow both localhost and container name for dev environments
        let localhost_url = self.keycloak_url.replace("keycloak:8180", "localhost:8180");
        validation.set_issuer(&[&self.keycloak_url, &localhost_url]);

        let token_data = decode::<Claims>(token, &key, &validation)
            .map_err(|e| format!("Token validation failed: {}", e))?;

        Ok(token_data.claims)
    }
}

pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = auth_state
        .validate_token(token)
        .await
        .map_err(|e| {
            tracing::error!("Token validation failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    // Insert claims into request extensions for handlers to access
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
