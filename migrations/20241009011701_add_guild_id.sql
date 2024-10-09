-- Add migration script here
ALTER TABLE meetups
ADD COLUMN guild_id INTEGER;

ALTER TABLE delays
ADD COLUMN guild_id INTEGER;
