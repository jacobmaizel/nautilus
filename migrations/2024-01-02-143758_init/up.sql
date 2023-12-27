-- Your SQL goes here

CREATE TYPE user_type AS ENUM ('trainer', 'client', 'user');

CREATE TABLE users (
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- General Info
    onboarding_completed BOOLEAN NOT NULL DEFAULT FALSE,
    user_type user_type NOT NULL DEFAULT 'user',
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    beta_access BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Personal Info
    first_name VARCHAR(255) NOT NULL,
    last_name  VARCHAR(255) NOT NULL,
    user_name VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    phone_number VARCHAR(255) NOT NULL DEFAULT '',
    image VARCHAR(255) NOT NULL DEFAULT '',
    birthday DATE,
    bio VARCHAR(255) NOT NULL DEFAULT '',
    gender VARCHAR(30) NOT NULL DEFAULT '',

    -- Provider Info
    provider_id VARCHAR(255) NOT NULL UNIQUE,

    -- Training stuff
    training_approach VARCHAR(255) NOT NULL DEFAULT '',
    training_years INT NOT NULL DEFAULT 0,
    training_specializations VARCHAR(255) NOT NULL DEFAULT '',

    -- Client stuff
    goals VARCHAR(255) NOT NULL DEFAULT '',
    weight INT NOT NULL DEFAULT 0
);
