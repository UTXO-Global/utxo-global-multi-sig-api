-- Add migration script here
ALTER TABLE multi_sig_signers ALTER COLUMN multi_sig_address TYPE TEXT;
ALTER TABLE multi_sig_info ALTER COLUMN multi_sig_address TYPE TEXT;
ALTER TABLE multi_sig_info ALTER COLUMN mutli_sig_witness_data TYPE TEXT;
ALTER TABLE multi_sig_invites ALTER COLUMN multi_sig_address TYPE TEXT;
ALTER TABLE multi_sig_info RENAME COLUMN mutli_sig_witness_data TO multi_sig_witness_data;