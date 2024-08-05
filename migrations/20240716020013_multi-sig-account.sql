-- Add migration script here

CREATE TABLE IF NOT EXISTS multi_sig_signers (
  id SERIAL PRIMARY KEY,
  multi_sig_address VARCHAR(200),
  signer_address VARCHAR(200) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX signer_address_index ON multi_sig_signers (signer_address);

CREATE TABLE IF NOT EXISTS multi_sig_info (
  multi_sig_address VARCHAR(200) PRIMARY KEY,
  threshold SMALLINT NOT NULL DEFAULT 1,
  signers SMALLINT NOT NULL DEFAULT 1,
  mutli_sig_witness_data VARCHAR(200) NOT NULL,
  name VARCHAR(200) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cells (
  multi_sig_address VARCHAR(200) PRIMARY KEY,
  outpoint VARCHAR(100) NOT NULL,
  transaction_id VARCHAR(66) NOT NULL,
  status SMALLINT NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  CONSTRAINT unique_outpoint UNIQUE (outpoint)
);

CREATE TABLE IF NOT EXISTS transactions (
  transaction_id VARCHAR(66) PRIMARY KEY,
  payload TEXT NOT NULL,
  status SMALLINT NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS signatures (
  signer_address VARCHAR(200) NOT NULL,
  transaction_id VARCHAR(66) NOT NULL,
  signature VARCHAR(130) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (signer_address, transaction_id)
);