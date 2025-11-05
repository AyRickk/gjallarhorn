use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_histogram_vec, register_int_gauge_vec, CounterVec,
    HistogramVec, IntGaugeVec, TextEncoder, Encoder,
};

lazy_static! {
    pub static ref FEEDBACK_COUNTER: CounterVec = register_counter_vec!(
        "feedback_total",
        "Total number of feedbacks submitted",
        &["service", "feedback_type"]
    )
    .unwrap();

    pub static ref FEEDBACK_RATING: HistogramVec = register_histogram_vec!(
        "feedback_rating",
        "Distribution of feedback ratings",
        &["service"],
        vec![1.0, 2.0, 3.0, 4.0, 5.0]
    )
    .unwrap();

    pub static ref FEEDBACK_THUMBS_UP: CounterVec = register_counter_vec!(
        "feedback_thumbs_up_total",
        "Total number of thumbs up",
        &["service"]
    )
    .unwrap();

    pub static ref FEEDBACK_THUMBS_DOWN: CounterVec = register_counter_vec!(
        "feedback_thumbs_down_total",
        "Total number of thumbs down",
        &["service"]
    )
    .unwrap();

    pub static ref FEEDBACK_COMMENTS: CounterVec = register_counter_vec!(
        "feedback_comments_total",
        "Total number of comments",
        &["service"]
    )
    .unwrap();

    pub static ref ACTIVE_USERS: IntGaugeVec = register_int_gauge_vec!(
        "feedback_active_users",
        "Number of active users providing feedback",
        &["service"]
    )
    .unwrap();

    pub static ref API_REQUESTS: CounterVec = register_counter_vec!(
        "feedback_api_requests_total",
        "Total number of API requests",
        &["method", "endpoint", "status"]
    )
    .unwrap();

    pub static ref API_LATENCY: HistogramVec = register_histogram_vec!(
        "feedback_api_latency_seconds",
        "API request latency in seconds",
        &["method", "endpoint"]
    )
    .unwrap();
}

pub fn record_feedback(service: &str, feedback_type: &str, rating: Option<i32>, thumbs_up: Option<bool>, has_comment: bool) {
    FEEDBACK_COUNTER
        .with_label_values(&[service, feedback_type])
        .inc();

    if let Some(rating) = rating {
        FEEDBACK_RATING
            .with_label_values(&[service])
            .observe(rating as f64);
    }

    if let Some(thumbs) = thumbs_up {
        if thumbs {
            FEEDBACK_THUMBS_UP
                .with_label_values(&[service])
                .inc();
        } else {
            FEEDBACK_THUMBS_DOWN
                .with_label_values(&[service])
                .inc();
        }
    }

    if has_comment {
        FEEDBACK_COMMENTS
            .with_label_values(&[service])
            .inc();
    }
}

pub fn gather_metrics() -> Result<String, Box<dyn std::error::Error>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

pub async fn initialize_metrics_from_db(db: &crate::db::Database) -> anyhow::Result<()> {
    use crate::models::FeedbackQuery;

    // Fetch all feedbacks from database
    let feedbacks = db.query_feedbacks(FeedbackQuery {
        service: None,
        feedback_type: None,
        user_id: None,
        from_date: None,
        to_date: None,
        limit: None,
        offset: None,
    }).await?;

    // Replay all feedbacks to metrics
    for feedback in feedbacks {
        record_feedback(
            &feedback.service,
            &format!("{:?}", feedback.feedback_type),
            feedback.rating,
            feedback.thumbs_up,
            feedback.comment.is_some(),
        );
    }

    tracing::info!("Metrics initialized from database with {} feedbacks", db.query_feedbacks(FeedbackQuery {
        service: None,
        feedback_type: None,
        user_id: None,
        from_date: None,
        to_date: None,
        limit: None,
        offset: None,
    }).await?.len());

    Ok(())
}
