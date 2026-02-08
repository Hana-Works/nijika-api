-- Create user_role enum
CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'user');

-- Add role column to users table with default value 'user'
ALTER TABLE users ADD COLUMN role user_role NOT NULL DEFAULT 'user';

-- Create index on role for faster filtering if needed
CREATE INDEX idx_users_role ON users(role);
