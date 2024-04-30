-- Your SQL goes here

ALTER TABLE programs
ADD COLUMN complete BOOLEAN NOT NULL DEFAULT false;
