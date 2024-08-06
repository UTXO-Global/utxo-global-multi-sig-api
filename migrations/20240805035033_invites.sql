-- Add migration script here

CREATE TABLE IF NOT EXISTS multi_sig_invites (
  id SERIAL PRIMARY KEY,
  multi_sig_address VARCHAR(200) NOT NULL,
  signer_address VARCHAR(200) NOT NULL,
  status SMALLINT DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX multi_sig_invites_signer_index ON multi_sig_invites (signer_address);