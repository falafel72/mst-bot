-- Add migration script here
CREATE TABLE meetups (
       user_id TEXT NOT NULL,
       datetime_unix INTEGER NOT NULL
);
CREATE TABLE delays (
       user_id TEXT NOT NULL,
       delay_seconds INTEGER NOT NULL
);
