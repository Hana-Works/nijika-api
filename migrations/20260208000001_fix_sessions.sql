-- Create the schema and table expected by tower-sessions-sqlx-store
CREATE SCHEMA IF NOT EXISTS tower_sessions;
CREATE TABLE IF NOT EXISTS tower_sessions.session (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);

-- Drop the old, unused sessions table if it exists
DROP TABLE IF EXISTS sessions;
