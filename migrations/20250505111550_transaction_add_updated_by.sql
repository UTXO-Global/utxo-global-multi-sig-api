-- Add migration script here
ALTER TABLE transactions ADD COLUMN updated_by VARCHAR(100);