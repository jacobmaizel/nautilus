-- Your SQL goes here

CREATE TABLE client_forms(
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Relationships
    client_id uuid REFERENCES clients(id) ON DELETE SET NULL,

    -- Fields
    health_history VARCHAR(500) NOT NULL DEFAULT '',
    lifestyle VARCHAR(500) NOT NULL DEFAULT '',
    time_availability VARCHAR(500) NOT NULL DEFAULT '',
    motivation VARCHAR(500) NOT NULL DEFAULT '',
    preferences VARCHAR(500) NOT NULL DEFAULT '',
    extra_details VARCHAR(500) NOT NULL DEFAULT ''
)

