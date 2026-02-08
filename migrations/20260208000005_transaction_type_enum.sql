-- Create transaction_type enum
CREATE TYPE transaction_type AS ENUM ('charge', 'refund', 'deposit', 'bonus');

-- Update transactions table to use the new enum
ALTER TABLE transactions 
  ALTER COLUMN type TYPE transaction_type 
  USING type::transaction_type;
