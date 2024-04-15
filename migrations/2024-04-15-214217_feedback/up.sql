-- Your SQL goes here


CREATE TABLE feedback (
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Relationships
    user_id uuid REFERENCES users(id) ON DELETE SET NULL,

    -- Fields
    title VARCHAR(100) NOT NULL DEFAULT '',
    description VARCHAR(500) NOT NULL DEFAULT '',
    followup BOOLEAN NOT NULL DEFAULT FALSE
)
