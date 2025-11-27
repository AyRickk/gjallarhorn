use axum::{
    extract::{ConnectInfo, Request},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::observability::RequestId;

pub async fn metrics_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let uri = req.uri().path().to_string();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status().as_u16().to_string();

    // Record metrics
    crate::metrics::API_REQUESTS
        .with_label_values(&[&method, &uri, &status])
        .inc();

    crate::metrics::API_LATENCY
        .with_label_values(&[&method, &uri])
        .observe(duration.as_secs_f64());

    response
}

// Rate limiter state: IP -> (request_count, window_start)
lazy_static! {
    static ref RATE_LIMIT_MAP: Arc<DashMap<String, (u32, Instant)>> =
        Arc::new(DashMap::new());
}

// General rate limiting middleware: 100 req/sec per IP
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let ip = addr.ip().to_string();
    let now = Instant::now();

    let mut entry = RATE_LIMIT_MAP.entry(ip.clone()).or_insert((0, now));
    let (count, window_start) = entry.value_mut();

    // Reset window if 1 second has passed
    if now.duration_since(*window_start) > Duration::from_secs(1) {
        *count = 0;
        *window_start = now;
    }

    // Check if limit exceeded (100 requests per second)
    if *count >= 100 {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.",
        ));
    }

    *count += 1;
    drop(entry);

    Ok(next.run(req).await)
}

// Stricter rate limiting for auth endpoints: 5 req/min per IP
pub async fn auth_rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let ip = format!("auth_{}", addr.ip());
    let now = Instant::now();

    let mut entry = RATE_LIMIT_MAP.entry(ip.clone()).or_insert((0, now));
    let (count, window_start) = entry.value_mut();

    // Reset window if 1 minute has passed
    if now.duration_since(*window_start) > Duration::from_secs(60) {
        *count = 0;
        *window_start = now;
    }

    // Check if limit exceeded (5 requests per minute)
    if *count >= 5 {
        tracing::warn!("Rate limit exceeded for auth endpoint from IP: {}", addr.ip());
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many login attempts. Please try again later.",
        ));
    }

    *count += 1;
    drop(entry);

    Ok(next.run(req).await)
}

/// Request logging middleware with correlation IDs
///
/// This middleware:
/// - Generates a unique request ID for each request
/// - Adds the request ID to response headers (X-Request-ID)
/// - Logs structured request/response information
/// - Tracks request duration
/// - Includes client IP and user agent
pub async fn request_logging_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let request_id = RequestId::new();

    // Extract request details
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let client_ip = addr.ip().to_string();
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // Log incoming request with structured fields
    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        client_ip = %client_ip,
        user_agent = %user_agent,
        "Incoming request"
    );

    // Process request
    let mut response = next.run(req).await;

    // Calculate duration
    let duration = start.elapsed();
    let status = response.status();

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id.to_string()) {
        response.headers_mut().insert("X-Request-ID", header_value);
    }

    // Log response with structured fields based on status
    if status.is_server_error() {
        tracing::error!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            "Request completed with server error"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            "Request completed with client error"
        );
    } else {
        tracing::info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            "Request completed successfully"
        );
    }

    response
}
