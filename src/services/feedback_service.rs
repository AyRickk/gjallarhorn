use crate::db::Database;
use crate::error::{AppError, Result};
use crate::models::{Feedback, FeedbackQuery, FeedbackStats, FeedbackSubmission};
use crate::validation::Validate;
use uuid::Uuid;

pub struct FeedbackService {
    db: Database,
}

impl FeedbackService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_feedback(
        &self,
        user_id: &str,
        user_email: Option<&str>,
        submission: FeedbackSubmission,
    ) -> Result<Feedback> {
        // Validate input
        submission.validate()?;

        // Create feedback in database
        let feedback = self
            .db
            .create_feedback(user_id, user_email, submission.clone())
            .await?;

        // Record metrics
        crate::metrics::record_feedback(
            &submission.service,
            &format!("{:?}", submission.feedback_type),
            submission.rating,
            submission.thumbs_up,
            submission.comment.is_some(),
        );

        Ok(feedback)
    }

    pub async fn get_feedback(&self, id: Uuid) -> Result<Feedback> {
        self.db
            .get_feedback(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Feedback with id {} not found", id)))
    }

    pub async fn query_feedbacks(&self, query: FeedbackQuery) -> Result<Vec<Feedback>> {
        self.db.query_feedbacks(query).await.map_err(Into::into)
    }

    pub async fn get_stats(&self, service: Option<&str>) -> Result<Vec<FeedbackStats>> {
        self.db.get_stats(service).await.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    // Note: These are unit tests that would require mocking the database
    // For now, we'll add integration tests separately

    #[test]
    fn test_service_creation() {
        // This is a simple test to verify the service can be created
        // In a real scenario, we'd use a mock database
    }
}
