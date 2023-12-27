-- Your SQL goes here

CREATE TYPE intensity_choices AS ENUM ('low', 'medium', 'high');

CREATE TABLE programs (
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Relationships
    owner_id uuid REFERENCES users(id) ON DELETE CASCADE,
    
    -- Fields
    name VARCHAR (50) NOT NULL,
    description VARCHAR (255) NOT NULL DEFAULT '',
    duration VARCHAR (50) NOT NULL DEFAULT '', 
    focus_areas VARCHAR(255) NOT NULL DEFAULT '',
    target_audience VARCHAR (50) NOT NULL DEFAULT '',
    program_image VARCHAR(255) NOT NULL DEFAULT '', 
    intensity intensity_choices NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE workouts (
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Relationships
    program_id uuid REFERENCES programs(id) ON DELETE CASCADE,
    owner_id uuid REFERENCES users(id) ON DELETE CASCADE,
    
    -- Fields
    name VARCHAR (50) NOT NULL,
    description VARCHAR (255) NOT NULL DEFAULT '',
    duration VARCHAR (50) NOT NULL DEFAULT '',
    sequence INT NOT NULL,
    week INT NOT NULL DEFAULT 1,
    intensity intensity_choices NOT NULL,
    workout_type VARCHAR (50) NOT NULL DEFAULT '',
    equipment_needed VARCHAR(255) NOT NULL DEFAULT '',
    image VARCHAR(255) NOT NULL DEFAULT '',
    video VARCHAR(255) NOT NULL DEFAULT '',
    template BOOLEAN NOT NULL DEFAULT FALSE,
    slug VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE exercises (
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Relationships
    workout_id uuid REFERENCES workouts(id) ON DELETE CASCADE,
    owner_id uuid REFERENCES users(id) ON DELETE CASCADE,

    -- Fields
    name VARCHAR (50) NOT NULL,
    description VARCHAR (255) NOT NULL DEFAULT '', 
    duration VARCHAR (50) NOT NULL DEFAULT '',
    reps VARCHAR (50) NOT NULL DEFAULT '',
    sets INT NOT NULL,
    rest_period VARCHAR (50) NOT NULL DEFAULT '',
    intensity intensity_choices NOT NULL DEFAULT 'low',
    equipment VARCHAR(255) NOT NULL DEFAULT '',
    muscle_groups VARCHAR(255) NOT NULL DEFAULT '',
    image VARCHAR(255) NOT NULL DEFAULT '',
    video VARCHAR(255) NOT NULL DEFAULT '',
    instructions VARCHAR(500) NOT NULL DEFAULT '',
    sequence INT NOT NULL DEFAULT 0,
    slug VARCHAR(255) NOT NULL UNIQUE
);


