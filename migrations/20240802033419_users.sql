-- Add migration script here

CREATE TABLE IF NOT EXISTS users (
  user_address VARCHAR(200) PRIMARY KEY,
  nonce VARCHAR(255),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

