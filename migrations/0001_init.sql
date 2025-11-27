-- Initial schema for ClewdR Kill Edition
-- Mirrors backend/db models and queries

-- Cookie queue table
CREATE TABLE IF NOT EXISTS cookies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cookie TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL CHECK (status IN ('pending', 'banned', 'checking')),
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    last_used DATETIME,
    request_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_cookies_status_created_at
    ON cookies (status, created_at);

CREATE INDEX IF NOT EXISTS idx_cookies_updated_at
    ON cookies (updated_at);

-- Aggregated statistics (persisted snapshots)
CREATE TABLE IF NOT EXISTS stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    total_requests INTEGER NOT NULL,
    success_count INTEGER NOT NULL,
    error_count INTEGER NOT NULL,
    avg_response_time REAL NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_stats_timestamp
    ON stats (timestamp);

-- Config KV store (persisted overrides)
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME NOT NULL
);
