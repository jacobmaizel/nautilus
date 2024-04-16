-- This file should undo anything in `up.sql`

ALTER TABLE clients
DROP COLUMN accepted_invite_at;
