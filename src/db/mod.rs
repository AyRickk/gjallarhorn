use crate::models::{Feedback, FeedbackQuery, FeedbackStats, FeedbackSubmission};
use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(50)
            .connect(database_url)
            .await
            .context("Failed to connect to database")?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("Failed to run migrations")?;
        Ok(())
    }

    pub async fn create_feedback(
        &self,
        user_id: &str,
        user_email: Option<&str>,
        submission: FeedbackSubmission,
    ) -> Result<Feedback> {
        let feedback = sqlx::query_as::<_, Feedback>(
            r#"
            INSERT INTO feedbacks (user_id, user_email, service, feedback_type, rating, thumbs_up, comment, context)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(user_email)
        .bind(submission.service)
        .bind(submission.feedback_type)
        .bind(submission.rating)
        .bind(submission.thumbs_up)
        .bind(submission.comment)
        .bind(submission.context)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create feedback")?;

        Ok(feedback)
    }

    pub async fn get_feedback(&self, id: uuid::Uuid) -> Result<Option<Feedback>> {
        let feedback = sqlx::query_as::<_, Feedback>(
            r#"
            SELECT * FROM feedbacks WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get feedback")?;

        Ok(feedback)
    }

    pub async fn query_feedbacks(&self, query: FeedbackQuery) -> Result<Vec<Feedback>> {
        let mut sql = String::from("SELECT * FROM feedbacks WHERE 1=1");
        let mut bind_count = 0;

        if query.service.is_some() {
            bind_count += 1;
            sql.push_str(&format!(" AND service = ${}", bind_count));
        }

        if query.feedback_type.is_some() {
            bind_count += 1;
            sql.push_str(&format!(" AND feedback_type = ${}", bind_count));
        }

        if query.user_id.is_some() {
            bind_count += 1;
            sql.push_str(&format!(" AND user_id = ${}", bind_count));
        }

        if query.from_date.is_some() {
            bind_count += 1;
            sql.push_str(&format!(" AND created_at >= ${}", bind_count));
        }

        if query.to_date.is_some() {
            bind_count += 1;
            sql.push_str(&format!(" AND created_at <= ${}", bind_count));
        }

        sql.push_str(" ORDER BY created_at DESC");

        if query.limit.is_some() {
            bind_count += 1;
            sql.push_str(&format!(" LIMIT ${}", bind_count));
        }

        if query.offset.is_some() {
            bind_count += 1;
            sql.push_str(&format!(" OFFSET ${}", bind_count));
        }

        let mut query_builder = sqlx::query_as::<_, Feedback>(&sql);

        if let Some(service) = &query.service {
            query_builder = query_builder.bind(service);
        }

        if let Some(feedback_type) = &query.feedback_type {
            query_builder = query_builder.bind(feedback_type);
        }

        if let Some(user_id) = &query.user_id {
            query_builder = query_builder.bind(user_id);
        }

        if let Some(from_date) = query.from_date {
            query_builder = query_builder.bind(from_date);
        }

        if let Some(to_date) = query.to_date {
            query_builder = query_builder.bind(to_date);
        }

        if let Some(limit) = query.limit {
            query_builder = query_builder.bind(limit);
        }

        if let Some(offset) = query.offset {
            query_builder = query_builder.bind(offset);
        }

        let feedbacks = query_builder
            .fetch_all(&self.pool)
            .await
            .context("Failed to query feedbacks")?;

        Ok(feedbacks)
    }

    pub async fn get_stats(&self, service: Option<&str>) -> Result<Vec<FeedbackStats>> {
        let stats = if let Some(service) = service {
            sqlx::query_as::<_, FeedbackStats>(
                r#"
                SELECT
                    service,
                    COUNT(*) as total_count,
                    CAST(AVG(CASE WHEN rating IS NOT NULL THEN rating END) AS float8) as rating_avg,
                    COUNT(CASE WHEN thumbs_up = true THEN 1 END)::bigint as thumbs_up_count,
                    COUNT(CASE WHEN thumbs_up = false THEN 1 END)::bigint as thumbs_down_count,
                    CASE
                        WHEN COUNT(CASE WHEN thumbs_up IS NOT NULL THEN 1 END) > 0
                        THEN COUNT(CASE WHEN thumbs_up = true THEN 1 END)::float / COUNT(CASE WHEN thumbs_up IS NOT NULL THEN 1 END)::float
                        ELSE NULL
                    END as thumbs_up_ratio,
                    COUNT(CASE WHEN comment IS NOT NULL THEN 1 END)::bigint as comment_count
                FROM feedbacks
                WHERE service = $1
                GROUP BY service
                "#,
            )
            .bind(service)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, FeedbackStats>(
                r#"
                SELECT
                    service,
                    COUNT(*) as total_count,
                    CAST(AVG(CASE WHEN rating IS NOT NULL THEN rating END) AS float8) as rating_avg,
                    COUNT(CASE WHEN thumbs_up = true THEN 1 END)::bigint as thumbs_up_count,
                    COUNT(CASE WHEN thumbs_up = false THEN 1 END)::bigint as thumbs_down_count,
                    CASE
                        WHEN COUNT(CASE WHEN thumbs_up IS NOT NULL THEN 1 END) > 0
                        THEN COUNT(CASE WHEN thumbs_up = true THEN 1 END)::float / COUNT(CASE WHEN thumbs_up IS NOT NULL THEN 1 END)::float
                        ELSE NULL
                    END as thumbs_up_ratio,
                    COUNT(CASE WHEN comment IS NOT NULL THEN 1 END)::bigint as comment_count
                FROM feedbacks
                GROUP BY service
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(stats)
    }

    pub async fn refresh_stats(&self) -> Result<()> {
        sqlx::query("SELECT refresh_feedback_stats()")
            .execute(&self.pool)
            .await
            .context("Failed to refresh stats")?;
        Ok(())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
