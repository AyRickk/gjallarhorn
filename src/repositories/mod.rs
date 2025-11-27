//! # Repositories Layer (Infrastructure Layer)
//!
//! This module implements the Repository pattern, providing an abstraction over data access.
//! Repositories encapsulate data persistence logic and allow the application layer to work
//! with domain objects without knowing the underlying data source.
//!
//! ## Design Pattern: Repository
//! - Trait-based abstraction (`FeedbackRepository`) defines the contract
//! - Concrete implementations (e.g., `PostgresFeedbackRepository`) handle specific databases
//! - Business logic (services) depends on traits, not concrete implementations
//!
//! ## Benefits
//! - **Testability**: Services can be tested with mock repositories
//! - **Flexibility**: Easy to swap PostgreSQL for another data source
//! - **Separation of Concerns**: Data access logic isolated from business logic
//! - **Type Safety**: Async traits ensure compile-time checking of data operations

mod feedback_repository;

pub use feedback_repository::{FeedbackRepository, PostgresFeedbackRepository};
