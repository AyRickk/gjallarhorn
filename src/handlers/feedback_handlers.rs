use crate::auth::Claims;
use crate::error::Result;
use crate::models::{FeedbackQuery, FeedbackResponse, FeedbackStats, FeedbackSubmission};
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;

use super::AppState;

// POST /api/v1/feedbacks - Submit a new feedback
pub async fn create_feedback(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(submission): Json<FeedbackSubmission>,
) -> Result<Json<FeedbackResponse>> {
    // Service layer handles all business logic including validation,
    // persistence, metrics recording, and webhook notifications
    let feedback = state
        .service
        .create_feedback(&claims.sub, claims.email.as_deref(), submission)
        .await?;

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
    // Apply default limit if not specified
    if query.limit.is_none() {
        query.limit = Some(100);
    }

    // Service layer handles validation
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
