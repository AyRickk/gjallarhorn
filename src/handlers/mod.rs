//! # Handlers Layer (Presentation Layer)
//!
//! This module contains HTTP request handlers that form the presentation layer.
//! Handlers are responsible for:
//! - Parsing HTTP requests and extracting parameters
//! - Delegating business logic to the service layer
//! - Transforming service responses into HTTP responses
//! - HTTP-specific concerns (status codes, headers, serialization)
//!
//! ## Design Principles
//! - **Thin Handlers**: Minimal logic - delegate to services immediately
//! - **HTTP-Only Concerns**: Only deal with HTTP-specific details here
//! - **No Business Logic**: Business rules belong in the service layer
//! - **Clear Responsibility**: Each handler maps to one HTTP endpoint
//!
//! ## Module Organization
//! - `auth_handlers`: Authentication endpoints (login)
//! - `feedback_handlers`: Core feedback CRUD operations
//! - `export_handlers`: Data export functionality
//! - `health_handlers`: Health checks and metrics

use crate::config::Config;
use crate::services::FeedbackService;
use std::sync::Arc;

// Handler modules
mod auth_handlers;
mod export_handlers;
mod feedback_handlers;
mod health_handlers;

// Re-export handler functions
pub use auth_handlers::{login, LoginRequest, LoginResponse};
pub use export_handlers::export_feedbacks;
pub use feedback_handlers::{create_feedback, get_feedback, get_stats, query_feedbacks};
pub use health_handlers::{health_check, metrics_handler};

// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub service: Arc<FeedbackService>,
    pub config: Arc<Config>,
}
