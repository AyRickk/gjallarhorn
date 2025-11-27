use crate::error::Result;
use crate::exports::export;
use crate::models::{ExportQuery, FeedbackQuery};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::AppState;

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
