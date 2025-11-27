use crate::error::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use super::AppState;

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

    // Check database connection via service
    let db_healthy = state.service.health_check().await.is_ok();

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
