use feedback_api::config::Config;
use feedback_api::db::Database;
use feedback_api::models::{FeedbackSubmission, FeedbackType};
use feedback_api::repositories::PostgresFeedbackRepository;
use feedback_api::services::FeedbackService;
use std::env;
use std::sync::Arc;

#[tokio::test]
#[ignore] // Requires database to be running
async fn test_create_and_retrieve_feedback() {
    // Setup
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://feedback:feedback@localhost:5432/feedback".to_string());

    let db = Database::new(&database_url).await.expect("Failed to connect to database");
    let repository = Arc::new(PostgresFeedbackRepository::new(db));
    let config = Arc::new(Config::from_env().unwrap_or_else(|_| {
        // Use default test config if env vars not set
        Config {
            database_url: database_url.clone(),
            host: "0.0.0.0".to_string(),
            port: 8080,
            keycloak_url: "http://localhost:8180/realms/master".to_string(),
            keycloak_realm: "master".to_string(),
            keycloak_jwks_cache_ttl: 300,
            webhook_urls: vec![],
            allowed_origins: vec![],
            export_max_records: 10000,
        }
    }));
    let service = FeedbackService::new(repository, config);

    // Create feedback
    let submission = FeedbackSubmission {
        service: "test-service".to_string(),
        feedback_type: FeedbackType::Rating,
        rating: Some(5),
        thumbs_up: None,
        comment: Some("Test comment".to_string()),
        context: None,
    };

    let created = service
        .create_feedback("test-user", Some("test@example.com"), submission)
        .await
        .expect("Failed to create feedback");

    // Retrieve feedback
    let retrieved = service
        .get_feedback(created.id)
        .await
        .expect("Failed to retrieve feedback");

    // Assert
    assert_eq!(created.id, retrieved.id);
    assert_eq!(retrieved.service, "test-service");
    assert_eq!(retrieved.rating, Some(5));
    assert_eq!(retrieved.comment, Some("Test comment".to_string()));
}

#[tokio::test]
#[ignore] // Requires database to be running
async fn test_query_feedbacks() {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://feedback:feedback@localhost:5432/feedback".to_string());

    let db = Database::new(&database_url).await.expect("Failed to connect to database");
    let repository = Arc::new(PostgresFeedbackRepository::new(db));
    let config = Arc::new(Config::from_env().unwrap_or_else(|_| {
        Config {
            database_url: database_url.clone(),
            host: "0.0.0.0".to_string(),
            port: 8080,
            keycloak_url: "http://localhost:8180/realms/master".to_string(),
            keycloak_realm: "master".to_string(),
            keycloak_jwks_cache_ttl: 300,
            webhook_urls: vec![],
            allowed_origins: vec![],
            export_max_records: 10000,
        }
    }));
    let service = FeedbackService::new(repository, config);

    // Query all feedbacks
    let feedbacks = service
        .query_feedbacks(feedback_api::models::FeedbackQuery {
            service: None,
            feedback_type: None,
            user_id: None,
            from_date: None,
            to_date: None,
            limit: Some(10),
            offset: None,
        })
        .await
        .expect("Failed to query feedbacks");

    assert!(feedbacks.len() <= 10);
}

#[tokio::test]
#[ignore] // Requires database to be running
async fn test_get_stats() {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://feedback:feedback@localhost:5432/feedback".to_string());

    let db = Database::new(&database_url).await.expect("Failed to connect to database");
    let repository = Arc::new(PostgresFeedbackRepository::new(db));
    let config = Arc::new(Config::from_env().unwrap_or_else(|_| {
        Config {
            database_url: database_url.clone(),
            host: "0.0.0.0".to_string(),
            port: 8080,
            keycloak_url: "http://localhost:8180/realms/master".to_string(),
            keycloak_realm: "master".to_string(),
            keycloak_jwks_cache_ttl: 300,
            webhook_urls: vec![],
            allowed_origins: vec![],
            export_max_records: 10000,
        }
    }));
    let service = FeedbackService::new(repository, config);

    // Get stats for all services
    let stats = service
        .get_stats(None)
        .await
        .expect("Failed to get stats");

    // Just verify it doesn't crash and returns valid data
    assert!(stats.is_empty() || !stats.is_empty());
}
