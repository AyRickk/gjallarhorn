//! # Observability Module (Infrastructure Layer)
//!
//! This module provides comprehensive observability features including:
//! - Structured logging with JSON output
//! - Distributed tracing with correlation IDs
//! - Request context propagation
//! - Performance tracking
//!
//! ## Design Principles
//! - Structured logs for easy parsing and analysis
//! - Correlation IDs for distributed tracing across requests
//! - Minimal performance impact
//! - Production-ready logging configuration

mod logging;
mod request_context;

pub use logging::init_logging;
pub use request_context::{RequestContext, RequestId};
