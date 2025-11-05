use feedback_api::auth::{auth_middleware, AuthState};
use feedback_api::config::Config;
use feedback_api::db::Database;
use feedback_api::handlers::{
    create_feedback, export_feedbacks, get_feedback, get_stats, health_check, login,
    metrics_handler, query_feedbacks, AppState,
};
use feedback_api::services::FeedbackService;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,feedback_api=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // Connect to database
    let db = Database::new(&config.database_url).await?;
    tracing::info!("Database connected successfully");

    // Run migrations
    db.run_migrations().await?;
    tracing::info!("Database migrations completed");

    // Initialize metrics from database
    feedback_api::metrics::initialize_metrics_from_db(&db).await?;
    tracing::info!("Metrics initialized from database");

    // Create auth state
    let auth_state = AuthState::new(
        config.keycloak_url.clone(),
        config.keycloak_realm.clone(),
        config.keycloak_jwks_cache_ttl,
    );

    // Create service layer
    let feedback_service = Arc::new(FeedbackService::new(db));

    // Create app state
    let app_state = AppState {
        service: feedback_service,
        config: Arc::new(config.clone()),
    };

    // Build protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/feedbacks", post(create_feedback))
        .route("/feedbacks", get(query_feedbacks))
        .route("/feedbacks/:id", get(get_feedback))
        .route("/feedbacks/stats", get(get_stats))
        .route("/feedbacks/export", get(export_feedbacks))
        .route_layer(axum::middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    // Build public routes
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/auth/login", post(login))
        .with_state(app_state.clone());

    // Combine all routes
    let app = Router::new()
        .nest("/api/v1", protected_routes)
        .merge(public_routes)
        .layer(axum::middleware::from_fn(feedback_api::middleware::metrics_middleware))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind(config.bind_address())
        .await?;

    tracing::info!("Server listening on {}", config.bind_address());

    axum::serve(listener, app)
        .await?;

    Ok(())
}
