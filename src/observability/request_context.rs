use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for tracing a request across the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(Uuid);

impl RequestId {
    /// Generate a new unique request ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Get the request ID as a string
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for RequestId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Request context containing metadata about the current request
///
/// This is used for:
/// - Distributed tracing across services
/// - Correlating logs for a single request
/// - Performance tracking
/// - Error tracking
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique identifier for this request
    pub request_id: RequestId,

    /// User ID if authenticated
    pub user_id: Option<String>,

    /// HTTP method
    pub method: String,

    /// Request path
    pub path: String,

    /// Client IP address
    pub client_ip: Option<String>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(method: String, path: String) -> Self {
        Self {
            request_id: RequestId::new(),
            user_id: None,
            method,
            path,
            client_ip: None,
        }
    }

    /// Set the user ID for this request
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the client IP for this request
    pub fn with_client_ip(mut self, client_ip: String) -> Self {
        self.client_ip = Some(client_ip);
        self
    }
}
