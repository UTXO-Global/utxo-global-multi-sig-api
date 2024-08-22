-- Add migration script here
CREATE TABLE IF NOT EXISTS bounty_contest_leaderboard (
  email VARCHAR(100) PRIMARY KEY,
  username VARCHAR(100),
  points SMALLINT NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);