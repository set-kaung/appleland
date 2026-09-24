-- Daily AI assistant usage: one row per admin per day. Rows are retained (no cleanup).
CREATE TABLE admin_ai_chat_usage (
    admin_id      UUID NOT NULL REFERENCES admins (id),
    usage_date    DATE NOT NULL,
    request_count INT  NOT NULL DEFAULT 0,
    PRIMARY KEY (admin_id, usage_date)
);