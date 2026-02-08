-- Ensure credits cannot be negative
ALTER TABLE users ADD CONSTRAINT check_credits_non_negative CHECK (credits >= 0);
