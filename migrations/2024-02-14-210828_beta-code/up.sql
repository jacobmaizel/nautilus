-- Your SQL goes here

CREATE TABLE betacode (
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Data
    code VARCHAR(50) NOT NULL UNIQUE
)
