-- Your SQL goes here
ALTER TABLE workouts
ADD COLUMN complete BOOLEAN NOT NULL DEFAULT false;
