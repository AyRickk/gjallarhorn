use feedback_api::auth::{auth_middleware, AuthState};
use feedback_api::config::Config;
use feedback_api::db::Database;
use feedback_api::handlers::{
    create_feedback, export_feedbacks, get_feedback, get_stats, health_check, login,
    metrics_handler, query_feedbacks, AppState,
};
use feedback_api::repositories::PostgresFeedbackRepository;
use feedback_api::services::FeedbackService;
use axum::{
    http::{header::{AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    feedback_api::observability::init_logging()?;

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // Connect to database
    let db = Database::new(&config.database_url).await?;
    tracing::info!("Database connected successfully");

    // Run migrations
    db.run_migrations().await?;
    tracing::info!("Database migrations completed");

    // Create repository layer
    let repository = Arc::new(PostgresFeedbackRepository::new(db));

    // Initialize metrics from database via repository
    feedback_api::metrics::initialize_metrics_from_db(repository.as_ref()).await?;
    tracing::info!("Metrics initialized from database");

    // Create auth state
    let auth_state = AuthState::new(
        config.keycloak_url.clone(),
        config.keycloak_realm.clone(),
        config.keycloak_jwks_cache_ttl,
    );

    // Create app state configuration
    let config_arc = Arc::new(config.clone());

    // Create service layer with repository and config
    let feedback_service = Arc::new(FeedbackService::new(repository, config_arc.clone()));

    // Create app state
    let app_state = AppState {
        service: feedback_service,
        config: config_arc,
    };

    // Build protected routes (require authentication + rate limiting)
    let protected_routes = Router::new()
        .route("/feedbacks", post(create_feedback))
        .route("/feedbacks", get(query_feedbacks))
        .route("/feedbacks/:id", get(get_feedback))
        .route("/feedbacks/stats", get(get_stats))
        .route("/feedbacks/export", get(export_feedbacks))
        .route_layer(axum::middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .layer(axum::middleware::from_fn(feedback_api::middleware::rate_limit_middleware));

    // Build public routes (health and metrics without rate limiting)
    let health_routes = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .with_state(app_state.clone());

    // Build auth routes with stricter rate limiting
    let auth_routes = Router::new()
        .route("/auth/login", post(login))
        .layer(axum::middleware::from_fn(feedback_api::middleware::auth_rate_limit_middleware))
        .with_state(app_state.clone());

    // Combine public and auth routes
    let public_routes = health_routes.merge(auth_routes);

    // Configure CORS with specific allowed origins
    let allowed_origins = config.allowed_origins.iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect::<Vec<_>>();

    let cors = if allowed_origins.is_empty() {
        tracing::warn!("No ALLOWED_ORIGINS configured, using permissive CORS (NOT RECOMMENDED FOR PRODUCTION)");
        CorsLayer::permissive()
    } else {
        tracing::info!("CORS configured with {} allowed origins", allowed_origins.len());
        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([AUTHORIZATION, CONTENT_TYPE])
            .allow_credentials(true)
            .max_age(Duration::from_secs(3600))
    };

    // Combine all routes
    let app = Router::new()
        .nest("/api/v1", protected_routes)
        .merge(public_routes)
        .layer(axum::middleware::from_fn(feedback_api::middleware::request_logging_middleware))
        .layer(axum::middleware::from_fn(feedback_api::middleware::metrics_middleware))
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB max request size
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    tracing::info!("Request body size limit set to 1MB");

    // Start server
    let listener = tokio::net::TcpListener::bind(config.bind_address())
        .await?;

    tracing::info!("Server listening on {}", config.bind_address());

    // Use into_make_service_with_connect_info to enable ConnectInfo extractor for rate limiting
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    tracing::info!("Server shutdown complete");

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown...");
        },
    }

    tracing::info!("Shutdown signal received, waiting for in-flight requests to complete...");
}
