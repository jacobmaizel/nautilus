-- This file should undo anything in `up.sql`

ALTER TABLE programs
DROP COLUMN template,
DROP COLUMN client_id;
