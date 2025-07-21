-- Add migration script here
ALTER TABLE multi_sig_signers ADD COLUMN signer_name VARCHAR(25);
ALTER TABLE multi_sig_invites ADD COLUMN signer_name VARCHAR(255);