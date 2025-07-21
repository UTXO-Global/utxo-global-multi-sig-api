-- Add migration script here
ALTER TABLE multi_sig_signers ADD COLUMN signer_name VARCHAR(255);
ALTER TABLE multi_sig_invites ADD COLUMN signer_name VARCHAR(255);

UPDATE multi_sig_signers mss
SET signer_name = ab.signer_name
FROM (
    SELECT signer_address, MAX(signer_name) AS signer_name
    FROM address_books
    GROUP BY signer_address
) ab
WHERE mss.signer_address = ab.signer_address;

UPDATE multi_sig_invites msi
SET signer_name = ab.signer_name
FROM (
    SELECT signer_address, MAX(signer_name) AS signer_name
    FROM address_books
    GROUP BY signer_address
) ab
WHERE msi.signer_address = ab.signer_address;