use crate::error::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use super::AppState;

// Authentication structures
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

// POST /auth/login - Login endpoint (proxy to Keycloak)
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response> {
    let client = reqwest::Client::new();

    let token_url = format!(
        "{}/protocol/openid-connect/token",
        state.config.keycloak_url
    );

    let params = [
        ("client_id", "admin-cli"),
        ("username", &payload.username),
        ("password", &payload.password),
        ("grant_type", "password"),
    ];

    let response = client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| crate::error::AppError::InternalError(format!("Failed to connect to Keycloak: {}", e)))?;

    if !response.status().is_success() {
        // Record failed authentication attempt
        crate::metrics::AUTH_ATTEMPTS
            .with_label_values(&["failed"])
            .inc();

        return Err(crate::error::AppError::AuthenticationError(
            "Invalid credentials".to_string(),
        ));
    }

    let token_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| crate::error::AppError::InternalError(format!("Failed to parse Keycloak response: {}", e)))?;

    let login_response = LoginResponse {
        access_token: token_data["access_token"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        token_type: token_data["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string(),
        expires_in: token_data["expires_in"].as_u64().unwrap_or(60),
    };

    // Record successful authentication
    crate::metrics::AUTH_ATTEMPTS
        .with_label_values(&["success"])
        .inc();

    Ok((StatusCode::OK, Json(login_response)).into_response())
}
