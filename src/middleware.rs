use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

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
