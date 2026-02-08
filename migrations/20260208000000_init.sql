-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    github_id TEXT UNIQUE,
    gitlab_id TEXT UNIQUE,
    email TEXT,
    username TEXT,
    credits DECIMAL(10, 2) DEFAULT 50.00,
    api_key TEXT UNIQUE NOT NULL,
    oauth_account_created_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create index on api_key for fast lookups
CREATE INDEX IF NOT EXISTS idx_users_api_key ON users(api_key);

-- Sessions table for tower-sessions-sqlx-store
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);
