use crate::error::{AppError, Result};
use crate::models::{FeedbackSubmission, FeedbackType};

pub trait Validate {
    fn validate(&self) -> Result<()>;
}

impl Validate for FeedbackSubmission {
    fn validate(&self) -> Result<()> {
        // Validate service name
        if self.service.is_empty() {
            return Err(AppError::ValidationError(
                "Service name cannot be empty".to_string(),
            ));
        }

        if self.service.len() > 100 {
            return Err(AppError::ValidationError(
                "Service name too long (max 100 characters)".to_string(),
            ));
        }

        // Validate rating based on feedback type
        match self.feedback_type {
            FeedbackType::Rating => {
                if let Some(rating) = self.rating {
                    if !(1..=5).contains(&rating) {
                        return Err(AppError::ValidationError(
                            "Rating must be between 1 and 5".to_string(),
                        ));
                    }
                } else {
                    return Err(AppError::ValidationError(
                        "Rating is required for Rating feedback type".to_string(),
                    ));
                }
            }
            FeedbackType::Nps => {
                if let Some(rating) = self.rating {
                    if !(0..=10).contains(&rating) {
                        return Err(AppError::ValidationError(
                            "NPS score must be between 0 and 10".to_string(),
                        ));
                    }
                } else {
                    return Err(AppError::ValidationError(
                        "Rating is required for NPS feedback type".to_string(),
                    ));
                }
            }
            FeedbackType::Thumbs => {
                if self.thumbs_up.is_none() {
                    return Err(AppError::ValidationError(
                        "thumbs_up is required for Thumbs feedback type".to_string(),
                    ));
                }
            }
            FeedbackType::Comment => {
                if self.comment.is_none() || self.comment.as_ref().unwrap().is_empty() {
                    return Err(AppError::ValidationError(
                        "Comment is required for Comment feedback type".to_string(),
                    ));
                }
            }
        }

        // Validate comment length if present
        if let Some(comment) = &self.comment {
            if comment.len() > 5000 {
                return Err(AppError::ValidationError(
                    "Comment too long (max 5000 characters)".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_rating_feedback() {
        let feedback = FeedbackSubmission {
            service: "test-service".to_string(),
            feedback_type: FeedbackType::Rating,
            rating: Some(5),
            thumbs_up: None,
            comment: None,
            context: None,
        };
        assert!(feedback.validate().is_ok());
    }

    #[test]
    fn test_invalid_rating_out_of_range() {
        let feedback = FeedbackSubmission {
            service: "test-service".to_string(),
            feedback_type: FeedbackType::Rating,
            rating: Some(6),
            thumbs_up: None,
            comment: None,
            context: None,
        };
        assert!(feedback.validate().is_err());
    }

    #[test]
    fn test_missing_rating_for_rating_type() {
        let feedback = FeedbackSubmission {
            service: "test-service".to_string(),
            feedback_type: FeedbackType::Rating,
            rating: None,
            thumbs_up: None,
            comment: None,
            context: None,
        };
        assert!(feedback.validate().is_err());
    }

    #[test]
    fn test_valid_nps_feedback() {
        let feedback = FeedbackSubmission {
            service: "test-service".to_string(),
            feedback_type: FeedbackType::Nps,
            rating: Some(9),
            thumbs_up: None,
            comment: None,
            context: None,
        };
        assert!(feedback.validate().is_ok());
    }

    #[test]
    fn test_invalid_nps_out_of_range() {
        let feedback = FeedbackSubmission {
            service: "test-service".to_string(),
            feedback_type: FeedbackType::Nps,
            rating: Some(11),
            thumbs_up: None,
            comment: None,
            context: None,
        };
        assert!(feedback.validate().is_err());
    }

    #[test]
    fn test_valid_thumbs_feedback() {
        let feedback = FeedbackSubmission {
            service: "test-service".to_string(),
            feedback_type: FeedbackType::Thumbs,
            rating: None,
            thumbs_up: Some(true),
            comment: None,
            context: None,
        };
        assert!(feedback.validate().is_ok());
    }

    #[test]
    fn test_missing_thumbs_for_thumbs_type() {
        let feedback = FeedbackSubmission {
            service: "test-service".to_string(),
            feedback_type: FeedbackType::Thumbs,
            rating: None,
            thumbs_up: None,
            comment: None,
            context: None,
        };
        assert!(feedback.validate().is_err());
    }

    #[test]
    fn test_empty_service_name() {
        let feedback = FeedbackSubmission {
            service: "".to_string(),
            feedback_type: FeedbackType::Rating,
            rating: Some(5),
            thumbs_up: None,
            comment: None,
            context: None,
        };
        assert!(feedback.validate().is_err());
    }

    #[test]
    fn test_comment_too_long() {
        let feedback = FeedbackSubmission {
            service: "test-service".to_string(),
            feedback_type: FeedbackType::Comment,
            rating: None,
            thumbs_up: None,
            comment: Some("x".repeat(5001)),
            context: None,
        };
        assert!(feedback.validate().is_err());
    }
}
