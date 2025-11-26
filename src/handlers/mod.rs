use crate::auth::Claims;
use crate::config::Config;
use crate::error::Result;
use crate::exports::{export, send_webhook, WebhookPayload};
use crate::models::{
    ExportQuery, FeedbackQuery, FeedbackResponse, FeedbackStats, FeedbackSubmission,
};
use crate::services::FeedbackService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<FeedbackService>,
    pub config: Arc<Config>,
}

// POST /api/v1/feedbacks - Submit a new feedback
pub async fn create_feedback(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(submission): Json<FeedbackSubmission>,
) -> Result<Json<FeedbackResponse>> {
    let feedback = state
        .service
        .create_feedback(&claims.sub, claims.email.as_deref(), submission)
        .await?;

    // Send webhooks asynchronously
    if !state.config.webhook_urls.is_empty() {
        let webhook_urls = state.config.webhook_urls.clone();
        let feedback_clone = feedback.clone();
        tokio::spawn(async move {
            let payload = WebhookPayload {
                event: "feedback.created".to_string(),
                feedback: feedback_clone,
            };
            if let Err(e) = send_webhook(&webhook_urls, payload).await {
                tracing::error!("Failed to send webhooks: {}", e);
            }
        });
    }

    Ok(Json(feedback.into()))
}

// GET /api/v1/feedbacks/:id - Get a specific feedback
pub async fn get_feedback(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<FeedbackResponse>> {
    let feedback = state.service.get_feedback(id).await?;
    Ok(Json(feedback.into()))
}

// GET /api/v1/feedbacks - Query feedbacks
pub async fn query_feedbacks(
    State(state): State<AppState>,
    Query(mut query): Query<FeedbackQuery>,
) -> Result<Json<Vec<FeedbackResponse>>> {
    // Validate query parameters
    use crate::validation::Validate;
    query.validate()?;

    // Apply default limit if not specified
    if query.limit.is_none() {
        query.limit = Some(100);
    }

    let feedbacks = state.service.query_feedbacks(query).await?;
    let responses: Vec<FeedbackResponse> = feedbacks.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

// GET /api/v1/feedbacks/stats - Get feedback statistics
pub async fn get_stats(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<FeedbackStats>>> {
    let service = params.get("service").and_then(|v| v.as_str());
    let stats = state.service.get_stats(service).await?;
    Ok(Json(stats))
}

// GET /api/v1/feedbacks/export - Export feedbacks
pub async fn export_feedbacks(
    State(state): State<AppState>,
    Query(query): Query<ExportQuery>,
) -> Result<Response> {
    let feedback_query = FeedbackQuery {
        service: query.service,
        feedback_type: None,
        user_id: None,
        from_date: query.from_date,
        to_date: query.to_date,
        limit: Some(state.config.export_max_records as i64),
        offset: None,
    };

    let feedbacks = state.service.query_feedbacks(feedback_query).await?;
    let content = export(&feedbacks, query.format.clone())?;

    let content_type = match query.format {
        crate::models::ExportFormat::Json => "application/json",
        crate::models::ExportFormat::Csv => "text/csv",
    };

    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, content_type)],
        content,
    )
        .into_response())
}

// GET /metrics - Prometheus metrics endpoint
pub async fn metrics_handler() -> Result<Response> {
    let metrics = crate::metrics::gather_metrics()?;
    Ok((
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        metrics,
    )
        .into_response())
}

// GET /health - Health check endpoint
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Response> {
    use serde_json::json;

    // Check database connection
    let db_healthy = state.service.db().health_check().await.is_ok();

    if !db_healthy {
        tracing::warn!("Health check failed: database is unhealthy");
    }

    let overall_status = if db_healthy { "healthy" } else { "unhealthy" };
    let status_code = if db_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = json!({
        "status": overall_status,
        "service": "feedback-api",
        "checks": {
            "database": if db_healthy { "healthy" } else { "unhealthy" }
        }
    });

    Ok((status_code, Json(response)).into_response())
}

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

    Ok((StatusCode::OK, Json(login_response)).into_response())
}
