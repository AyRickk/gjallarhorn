use crate::models::{Feedback, ExportFormat};
use anyhow::Result;
use csv::Writer;

pub fn export_to_json(feedbacks: &[Feedback]) -> Result<String> {
    Ok(serde_json::to_string_pretty(feedbacks)?)
}

pub fn export_to_csv(feedbacks: &[Feedback]) -> Result<String> {
    let mut wtr = Writer::from_writer(vec![]);

    // Write headers
    wtr.write_record(&[
        "id",
        "user_id",
        "user_email",
        "service",
        "feedback_type",
        "rating",
        "thumbs_up",
        "comment",
        "context",
        "created_at",
    ])?;

    // Write data
    for feedback in feedbacks {
        wtr.write_record(&[
            feedback.id.to_string(),
            feedback.user_id.clone(),
            feedback.user_email.clone().unwrap_or_default(),
            feedback.service.clone(),
            format!("{:?}", feedback.feedback_type),
            feedback.rating.map(|r| r.to_string()).unwrap_or_default(),
            feedback.thumbs_up.map(|t| t.to_string()).unwrap_or_default(),
            feedback.comment.clone().unwrap_or_default(),
            feedback.context.as_ref().map(|c| c.to_string()).unwrap_or_default(),
            feedback.created_at.to_rfc3339(),
        ])?;
    }

    Ok(String::from_utf8(wtr.into_inner()?)?)
}

pub fn export(feedbacks: &[Feedback], format: ExportFormat) -> Result<String> {
    match format {
        ExportFormat::Json => export_to_json(feedbacks),
        ExportFormat::Csv => export_to_csv(feedbacks),
    }
}

#[derive(Debug, serde::Serialize)]
pub struct WebhookPayload {
    pub event: String,
    pub feedback: Feedback,
}

pub async fn send_webhook(urls: &[String], payload: WebhookPayload) -> Result<()> {
    let client = reqwest::Client::new();

    for url in urls {
        if let Err(e) = client
            .post(url)
            .json(&payload)
            .send()
            .await
        {
            tracing::error!("Failed to send webhook to {}: {}", url, e);
        } else {
            tracing::info!("Webhook sent successfully to {}", url);
        }
    }

    Ok(())
}
