-- Add migration script here
ALTER TABLE meetups
DROP COLUMN guild_id;

ALTER TABLE meetups
ADD COLUMN guild_id TEXT;

ALTER TABLE delays
DROP COLUMN guild_id;

ALTER TABLE delays
ADD COLUMN guild_id TEXT;
