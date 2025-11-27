// Library interface for feedback-api
// Exposes modules for testing and potential library usage
//
// # Architecture
//
// This application follows Clean Architecture principles with clear separation of concerns:
//
// ## Domain Layer (Core Business Logic)
// - `models`: Core domain entities and value objects (Feedback, FeedbackType, etc.)
// - `error`: Domain errors and result types
//
// ## Application Layer (Use Cases & Orchestration)
// - `services`: Business logic orchestration and use cases
// - `validation`: Input validation rules
//
// ## Infrastructure Layer (External Concerns)
// - `repositories`: Data access abstraction (Repository pattern with traits)
// - `db`: PostgreSQL database implementation
// - `auth`: JWT authentication with Keycloak
// - `exports`: Export functionality (CSV, JSON)
// - `metrics`: Prometheus metrics collection
// - `middleware`: HTTP middleware (rate limiting, metrics tracking)
//
// ## Presentation Layer (HTTP Interface)
// - `handlers`: HTTP request handlers organized by domain
//
// ## Cross-Cutting Concerns
// - `config`: Application configuration
//
// ## Dependency Flow
// Domain ← Application ← Infrastructure ← Presentation
//
// Each layer only depends on layers below it, ensuring clean separation and testability.

// Domain Layer
pub mod error;
pub mod models;

// Application Layer
pub mod services;
pub mod validation;

// Infrastructure Layer
pub mod auth;
pub mod db;
pub mod exports;
pub mod metrics;
pub mod middleware;
pub mod observability;
pub mod repositories;

// Presentation Layer
pub mod handlers;

// Cross-Cutting
pub mod config;
