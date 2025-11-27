use crate::config::Config;
use crate::error::{AppError, Result};
use crate::exports::{send_webhook, WebhookPayload};
use crate::models::{Feedback, FeedbackQuery, FeedbackStats, FeedbackSubmission};
use crate::repositories::FeedbackRepository;
use crate::validation::Validate;
use std::sync::Arc;
use uuid::Uuid;

/// Service layer for feedback operations
/// Handles business logic, orchestration, and coordination between components
pub struct FeedbackService {
    repository: Arc<dyn FeedbackRepository>,
    config: Arc<Config>,
}

impl FeedbackService {
    pub fn new(repository: Arc<dyn FeedbackRepository>, config: Arc<Config>) -> Self {
        Self { repository, config }
    }

    /// Health check - verify the service and its dependencies are accessible
    pub async fn health_check(&self) -> Result<()> {
        self.repository.health_check().await.map_err(Into::into)
    }

    /// Create a new feedback with full business logic orchestration
    /// This includes validation, persistence, metrics recording, and webhook notifications
    pub async fn create_feedback(
        &self,
        user_id: &str,
        user_email: Option<&str>,
        submission: FeedbackSubmission,
    ) -> Result<Feedback> {
        // Log with structured context
        tracing::debug!(
            user_id = %user_id,
            service = %submission.service,
            feedback_type = ?submission.feedback_type,
            has_comment = submission.comment.is_some(),
            "Creating feedback"
        );

        // 1. Validate input according to business rules
        self.validate_feedback_submission(&submission)?;

        // 2. Persist feedback via repository
        let feedback = self
            .repository
            .create(user_id, user_email, submission.clone())
            .await?;

        // Log successful creation with feedback ID
        tracing::info!(
            feedback_id = %feedback.id,
            user_id = %user_id,
            service = %feedback.service,
            feedback_type = ?feedback.feedback_type,
            "Feedback created successfully"
        );

        // 3. Record metrics asynchronously (fire and forget)
        self.record_feedback_metrics(&submission);

        // 4. Send webhook notifications asynchronously if configured
        self.trigger_webhook_notifications(feedback.clone()).await;

        Ok(feedback)
    }

    /// Get a specific feedback by ID
    pub async fn get_feedback(&self, id: Uuid) -> Result<Feedback> {
        self.repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Feedback with id {} not found", id)))
    }

    /// Query feedbacks with validation
    pub async fn query_feedbacks(&self, query: FeedbackQuery) -> Result<Vec<Feedback>> {
        // Validate query parameters
        query.validate()?;

        self.repository.query(query).await.map_err(Into::into)
    }

    /// Get aggregated statistics for a service
    pub async fn get_stats(&self, service: Option<&str>) -> Result<Vec<FeedbackStats>> {
        self.repository.get_stats(service).await.map_err(Into::into)
    }

    /// Get statistics for a specific service with additional validation
    pub async fn get_service_stats(&self, service: &str) -> Result<FeedbackStats> {
        // Validate service name is not empty
        if service.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Service name cannot be empty".to_string(),
            ));
        }

        let stats = self.repository.get_stats(Some(service)).await?;

        stats
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound(format!("No statistics found for service '{}'", service)))
    }

    // Private helper methods for business logic

    /// Validate feedback submission according to business rules
    fn validate_feedback_submission(&self, submission: &FeedbackSubmission) -> Result<()> {
        // Standard validation
        submission.validate()?;

        // Additional business rules
        // Rule: Service name should not be empty or just whitespace
        if submission.service.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Service name cannot be empty".to_string(),
            ));
        }

        // Rule: If rating is provided, it should match the feedback type
        if submission.rating.is_some() {
            use crate::models::FeedbackType;
            match submission.feedback_type {
                FeedbackType::Rating | FeedbackType::Nps => {
                    // Valid - these types can have ratings
                }
                _ => {
                    return Err(AppError::ValidationError(
                        format!("Rating is not applicable for feedback type {:?}", submission.feedback_type),
                    ));
                }
            }
        }

        // Rule: Thumbs up/down should only be present for Thumbs feedback type
        if submission.thumbs_up.is_some() {
            use crate::models::FeedbackType;
            if !matches!(submission.feedback_type, FeedbackType::Thumbs) {
                return Err(AppError::ValidationError(
                    format!("Thumbs up/down is not applicable for feedback type {:?}", submission.feedback_type),
                ));
            }
        }

        Ok(())
    }

    /// Record metrics for a feedback submission
    fn record_feedback_metrics(&self, submission: &FeedbackSubmission) {
        crate::metrics::record_feedback(
            &submission.service,
            &format!("{:?}", submission.feedback_type),
            submission.rating,
            submission.thumbs_up,
            submission.comment.is_some(),
        );
    }

    /// Trigger webhook notifications asynchronously
    async fn trigger_webhook_notifications(&self, feedback: Feedback) {
        if !self.config.webhook_urls.is_empty() {
            let webhook_urls = self.config.webhook_urls.clone();
            tokio::spawn(async move {
                let payload = WebhookPayload {
                    event: "feedback.created".to_string(),
                    feedback,
                };
                if let Err(e) = send_webhook(&webhook_urls, payload).await {
                    tracing::error!("Failed to send webhooks: {}", e);
                }
            });
        }
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
