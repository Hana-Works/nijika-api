-- Consolidated initial migration

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    github_id TEXT UNIQUE,
    gitlab_id TEXT UNIQUE,
    email TEXT,
    username TEXT,
    credits DECIMAL(16, 8) DEFAULT 50.00000000,
    api_key TEXT UNIQUE NOT NULL,
    oauth_account_created_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    avatar_url TEXT
);

-- Create index on api_key for fast lookups
CREATE INDEX IF NOT EXISTS idx_users_api_key ON users(api_key);

-- Schema and table for tower-sessions
CREATE SCHEMA IF NOT EXISTS tower_sessions;
CREATE TABLE IF NOT EXISTS tower_sessions.sessions (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);

-- Public sessions table (just in case)
CREATE TABLE IF NOT EXISTS public.sessions (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);