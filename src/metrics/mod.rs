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
    // Fetch aggregated metrics from database instead of loading all feedbacks
    let aggregates = db.get_metrics_aggregates().await?;

    let aggregate_count = aggregates.len();
    let mut total_feedbacks = 0i64;

    // Initialize metrics from aggregated data
    for agg in aggregates {
        let feedback_type_str = format!("{:?}", agg.feedback_type);

        // Set feedback counter
        FEEDBACK_COUNTER
            .with_label_values(&[&agg.service, &feedback_type_str])
            .inc_by(agg.total_count as f64);

        // Set rating histogram with individual observations from sum
        // Note: We can't restore exact individual ratings, but we can observe the average
        if let Some(rating_sum) = agg.rating_sum {
            if agg.total_count > 0 {
                let avg_rating = rating_sum / agg.total_count as f64;
                // Observe the average rating for each count
                // This approximates the distribution
                for _ in 0..agg.total_count {
                    FEEDBACK_RATING
                        .with_label_values(&[&agg.service])
                        .observe(avg_rating);
                }
            }
        }

        // Set thumbs counters
        if agg.thumbs_up_count > 0 {
            FEEDBACK_THUMBS_UP
                .with_label_values(&[&agg.service])
                .inc_by(agg.thumbs_up_count as f64);
        }

        if agg.thumbs_down_count > 0 {
            FEEDBACK_THUMBS_DOWN
                .with_label_values(&[&agg.service])
                .inc_by(agg.thumbs_down_count as f64);
        }

        // Set comments counter
        if agg.comment_count > 0 {
            FEEDBACK_COMMENTS
                .with_label_values(&[&agg.service])
                .inc_by(agg.comment_count as f64);
        }

        total_feedbacks += agg.total_count;
    }

    tracing::info!("Metrics initialized from database aggregates ({} total feedbacks across {} service/type combinations)",
        total_feedbacks, aggregate_count);

    Ok(())
}
