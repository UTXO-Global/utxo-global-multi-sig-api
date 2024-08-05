-- Add migration script here
CREATE TABLE IF NOT EXISTS address_books (
  id SERIAL PRIMARY KEY,
  user_address VARCHAR(200) NOT NULL,
  signer_address VARCHAR(200) NOT NULL,
  signer_name VARCHAR(255),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX address_books_user_index ON address_books (user_address);
CREATE INDEX address_books_signer_index ON address_books (signer_address);