-- Add migration script here
CREATE TABLE IF NOT EXISTS transaction_errors (
  id SERIAL PRIMARY KEY,
  transaction_id VARCHAR(100),
  signer_address VARCHAR(100),
  errors TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS transaction_rejects (
  transaction_id VARCHAR(100),
  signer_address VARCHAR(100),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (transaction_id, signer_address)
);