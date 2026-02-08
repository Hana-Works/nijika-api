-- Update credits column to higher precision
ALTER TABLE users ALTER COLUMN credits TYPE DECIMAL(16, 8);
ALTER TABLE users ALTER COLUMN credits SET DEFAULT 50.00000000;
