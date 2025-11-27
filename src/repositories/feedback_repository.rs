use crate::db::Database;
use crate::models::{Feedback, FeedbackQuery, FeedbackStats, FeedbackSubmission, MetricsAggregate};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for feedback operations
/// This abstraction allows for different implementations (PostgreSQL, in-memory, etc.)
/// and makes the code more testable
#[async_trait]
pub trait FeedbackRepository: Send + Sync {
    /// Create a new feedback
    async fn create(
        &self,
        user_id: &str,
        user_email: Option<&str>,
        submission: FeedbackSubmission,
    ) -> Result<Feedback>;

    /// Get a feedback by ID
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Feedback>>;

    /// Query feedbacks with filters
    async fn query(&self, query: FeedbackQuery) -> Result<Vec<Feedback>>;

    /// Get statistics for feedbacks
    async fn get_stats(&self, service: Option<&str>) -> Result<Vec<FeedbackStats>>;

    /// Get aggregated metrics for Prometheus initialization
    async fn get_metrics_aggregates(&self) -> Result<Vec<MetricsAggregate>>;

    /// Health check - verify repository is accessible
    async fn health_check(&self) -> Result<()>;
}

/// PostgreSQL implementation of FeedbackRepository
pub struct PostgresFeedbackRepository {
    db: Database,
}

impl PostgresFeedbackRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FeedbackRepository for PostgresFeedbackRepository {
    async fn create(
        &self,
        user_id: &str,
        user_email: Option<&str>,
        submission: FeedbackSubmission,
    ) -> Result<Feedback> {
        self.db.create_feedback(user_id, user_email, submission).await
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Feedback>> {
        self.db.get_feedback(id).await
    }

    async fn query(&self, query: FeedbackQuery) -> Result<Vec<Feedback>> {
        self.db.query_feedbacks(query).await
    }

    async fn get_stats(&self, service: Option<&str>) -> Result<Vec<FeedbackStats>> {
        self.db.get_stats(service).await
    }

    async fn get_metrics_aggregates(&self) -> Result<Vec<MetricsAggregate>> {
        self.db.get_metrics_aggregates().await
    }

    async fn health_check(&self) -> Result<()> {
        self.db.health_check().await
    }
}
