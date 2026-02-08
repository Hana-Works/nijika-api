-- Add is_active column to users table
ALTER TABLE users ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;

-- Create index for faster filtering of active users
CREATE INDEX idx_users_is_active ON users(is_active);
