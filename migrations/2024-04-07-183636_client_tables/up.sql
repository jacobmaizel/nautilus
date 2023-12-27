-- Your SQL goes here

CREATE TYPE invite_states AS ENUM ('pending','rejected','accepted');

CREATE TABLE clients (
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Relationships
    user_id uuid REFERENCES users(id) ON DELETE CASCADE,
    trainer_id uuid REFERENCES users(id) ON DELETE CASCADE,
    
    -- Data
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(50) NOT NULL DEFAULT '',
    -- pending, rejected, accepted
    invite invite_states NOT NULL DEFAULT 'pending'

    -- Constraints
    -- client user cant be clients to multiple trainers
    -- UNIQUE(user_id, trainer_id)
)

