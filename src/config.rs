use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub keycloak_url: String,
    pub keycloak_realm: String,
    pub keycloak_jwks_cache_ttl: u64,
    pub webhook_urls: Vec<String>,
    pub export_max_records: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .context("Invalid PORT")?;

        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;

        let keycloak_url = std::env::var("KEYCLOAK_URL")
            .context("KEYCLOAK_URL must be set")?;

        let keycloak_realm = std::env::var("KEYCLOAK_REALM")
            .unwrap_or_else(|_| "master".to_string());

        let keycloak_jwks_cache_ttl = std::env::var("KEYCLOAK_JWKS_CACHE_TTL")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .unwrap_or(3600);

        let webhook_urls = std::env::var("WEBHOOK_URLS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let export_max_records = std::env::var("EXPORT_MAX_RECORDS")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .unwrap_or(10000);

        Ok(Config {
            host,
            port,
            database_url,
            keycloak_url,
            keycloak_realm,
            keycloak_jwks_cache_ttl,
            webhook_urls,
            export_max_records,
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
