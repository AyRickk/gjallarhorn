use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "feedback_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum FeedbackType {
    Rating,    // 1-5 stars
    Thumbs,    // up/down
    Comment,   // text comment
    Nps,       // Net Promoter Score 0-10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSubmission {
    pub service: String,           // e.g., "visio", "chatbot", "console"
    pub feedback_type: FeedbackType,
    pub rating: Option<i32>,       // For rating (1-5) or NPS (0-10)
    pub thumbs_up: Option<bool>,   // For thumbs feedback
    pub comment: Option<String>,   // Optional comment
    pub context: Option<JsonValue>, // Flexible context (call_id, message_id, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Feedback {
    pub id: Uuid,
    pub user_id: String,           // From JWT
    pub user_email: Option<String>, // From JWT
    pub service: String,
    pub feedback_type: FeedbackType,
    pub rating: Option<i32>,
    pub thumbs_up: Option<bool>,
    pub comment: Option<String>,
    pub context: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackResponse {
    pub id: Uuid,
    pub service: String,
    pub feedback_type: FeedbackType,
    pub rating: Option<i32>,
    pub thumbs_up: Option<bool>,
    pub comment: Option<String>,
    pub context: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackQuery {
    pub service: Option<String>,
    pub feedback_type: Option<FeedbackType>,
    pub user_id: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeedbackStats {
    pub service: String,
    pub total_count: i64,
    pub rating_avg: Option<f64>,
    pub thumbs_up_count: i64,
    pub thumbs_down_count: i64,
    pub thumbs_up_ratio: Option<f64>,
    pub comment_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportQuery {
    pub format: ExportFormat,
    pub service: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MetricsAggregate {
    pub service: String,
    pub feedback_type: FeedbackType,
    pub total_count: i64,
    pub rating_sum: Option<f64>,
    pub thumbs_up_count: i64,
    pub thumbs_down_count: i64,
    pub comment_count: i64,
}

impl From<Feedback> for FeedbackResponse {
    fn from(feedback: Feedback) -> Self {
        FeedbackResponse {
            id: feedback.id,
            service: feedback.service,
            feedback_type: feedback.feedback_type,
            rating: feedback.rating,
            thumbs_up: feedback.thumbs_up,
            comment: feedback.comment,
            context: feedback.context,
            created_at: feedback.created_at,
        }
    }
}
