//! # Services Layer (Application Layer)
//!
//! This module contains the application's business logic and use case orchestration.
//! Services coordinate between the presentation layer (handlers) and the infrastructure
//! layer (repositories), implementing business rules and workflows.
//!
//! ## Responsibilities
//! - Business logic validation beyond basic input validation
//! - Use case orchestration (combining multiple repository operations)
//! - Cross-cutting concerns coordination (metrics, webhooks, events)
//! - Domain-specific error handling and transformation
//!
//! ## Design Principles
//! - Services depend on repository abstractions (traits), not concrete implementations
//! - Services are stateless and thread-safe (can be wrapped in Arc)
//! - Business logic lives here, not in handlers or repositories
//! - Each service method represents a complete use case or business operation

pub mod feedback_service;

pub use feedback_service::FeedbackService;
