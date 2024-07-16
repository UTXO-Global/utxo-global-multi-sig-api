-- Add migration script here

CREATE TABLE IF NOT EXISTS multi_sig_signers (
  multi_sig_address VARCHAR(200) PRIMARY KEY,
  signer_address VARCHAR(200) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX signer_address_index ON signers (signer_address);

CREATE TABLE IF NOT EXISTS multi_sig_info (
  multi_sig_address VARCHAR(200) PRIMARY KEY,
  threshold SMALLINT NOT NULL DEFAULT 1,
  signers SMALLINT NOT NULL DEFAULT 1,
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
  signature VARCHAR(200) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (signer_address, transaction_id)
);