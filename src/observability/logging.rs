use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize structured logging for the application
///
/// This configures tracing with:
/// - JSON structured output for production
/// - Environment-based log level filtering
/// - Contextual fields (timestamp, level, target, message)
pub fn init_logging() -> anyhow::Result<()> {
    // Determine log format based on environment
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string());

    // Configure filter from environment or use default
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,feedback_api=debug,sqlx=warn"));

    // Build subscriber based on format
    match log_format.as_str() {
        "json" => {
            // JSON format for production - structured logging
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_current_span(true)
                        .with_span_list(true)
                        .with_target(true)
                        .with_thread_ids(false)
                        .with_thread_names(false)
                        .with_file(false)
                        .with_line_number(false),
                )
                .init();
        }
        "pretty" | "human" => {
            // Pretty format for development - human-readable
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .pretty()
                        .with_target(true)
                        .with_thread_ids(false)
                        .with_thread_names(false),
                )
                .init();
        }
        _ => {
            // Compact format as fallback
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .init();
        }
    }

    tracing::info!(
        log_format = %log_format,
        "Logging initialized"
    );

    Ok(())
}
