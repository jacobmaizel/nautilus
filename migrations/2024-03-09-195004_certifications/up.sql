-- Your SQL goes here


CREATE TABLE certifications (
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Relationships
    user_id uuid REFERENCES users(id) ON DELETE CASCADE,
    
    -- Data
    name VARCHAR(50) NOT NULL,
    expiration DATE,

    UNIQUE(user_id, name)
)
