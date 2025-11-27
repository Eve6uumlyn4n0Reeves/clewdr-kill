-- Add cooldown and rate-limit tracking to cookies
ALTER TABLE cookies
ADD COLUMN next_retry_at DATETIME;

ALTER TABLE cookies
ADD COLUMN last_rate_limited_at DATETIME;
