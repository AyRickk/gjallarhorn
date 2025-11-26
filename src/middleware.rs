use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

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
