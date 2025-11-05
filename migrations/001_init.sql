-- Create feedback_type enum
CREATE TYPE feedback_type AS ENUM ('rating', 'thumbs', 'comment', 'nps');

-- Create feedbacks table
CREATE TABLE feedbacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    user_email VARCHAR(255),
    service VARCHAR(100) NOT NULL,
    feedback_type feedback_type NOT NULL,
    rating INTEGER,
    thumbs_up BOOLEAN,
    comment TEXT,
    context JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for common queries
CREATE INDEX idx_feedbacks_user_id ON feedbacks(user_id);
CREATE INDEX idx_feedbacks_service ON feedbacks(service);
CREATE INDEX idx_feedbacks_feedback_type ON feedbacks(feedback_type);
CREATE INDEX idx_feedbacks_created_at ON feedbacks(created_at DESC);
CREATE INDEX idx_feedbacks_service_created_at ON feedbacks(service, created_at DESC);

-- Create index on JSONB context for faster queries
CREATE INDEX idx_feedbacks_context ON feedbacks USING GIN(context);

-- Add constraints
ALTER TABLE feedbacks
    ADD CONSTRAINT chk_rating CHECK (rating IS NULL OR (rating >= 0 AND rating <= 10)),
    ADD CONSTRAINT chk_feedback_data CHECK (
        (feedback_type = 'rating' AND rating IS NOT NULL) OR
        (feedback_type = 'thumbs' AND thumbs_up IS NOT NULL) OR
        (feedback_type = 'comment' AND comment IS NOT NULL) OR
        (feedback_type = 'nps' AND rating IS NOT NULL)
    );

-- Create function to update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for updated_at
CREATE TRIGGER update_feedbacks_updated_at
    BEFORE UPDATE ON feedbacks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create materialized view for statistics (for better performance on large datasets)
CREATE MATERIALIZED VIEW feedback_stats AS
SELECT
    service,
    feedback_type,
    DATE_TRUNC('day', created_at) as date,
    COUNT(*) as count,
    AVG(rating) as avg_rating,
    COUNT(CASE WHEN thumbs_up = true THEN 1 END) as thumbs_up_count,
    COUNT(CASE WHEN thumbs_up = false THEN 1 END) as thumbs_down_count
FROM feedbacks
GROUP BY service, feedback_type, DATE_TRUNC('day', created_at);

-- Create index on materialized view
CREATE INDEX idx_feedback_stats_service_date ON feedback_stats(service, date DESC);

-- Create function to refresh stats
CREATE OR REPLACE FUNCTION refresh_feedback_stats()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY feedback_stats;
END;
$$ LANGUAGE plpgsql;
