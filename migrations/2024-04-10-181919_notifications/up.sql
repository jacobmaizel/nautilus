-- Your SQL goes here


CREATE TABLE notifications (
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Relationships
    sender_id uuid REFERENCES users(id) ON DELETE SET NULL,
    user_id uuid REFERENCES users(id) ON DELETE SET NULL,

    -- Fields
    title VARCHAR(50) NOT NULL DEFAULT '',
    content VARCHAR(255) NOT NULL DEFAULT '',
    category VARCHAR(50) NOT NULL DEFAULT '',
    status VARCHAR(50) NOT NULL DEFAULT '',
    opened_at TIMESTAMPTZ,
    data JSONB
)
