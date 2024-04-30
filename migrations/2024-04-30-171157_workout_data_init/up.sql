-- Your SQL goes here

CREATE TABLE workout_data(
    -- Meta
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Relationships
    workout_id uuid REFERENCES workouts(id) ON DELETE CASCADE,

    -- Fields
    hk_workout_id VARCHAR(50) NOT NULL DEFAULT '',

    hk_location_type VARCHAR(50) NOT NULL DEFAULT '', -- indoor / outdoor
    hk_workout_activity_type VARCHAR(50) NOT NULL DEFAULT '',

    hk_workout_duration_secs VARCHAR(50) NOT NULL DEFAULT '',

    hk_workout_start_date TIMESTAMPTZ,
    hk_workout_end_date TIMESTAMPTZ,

    hk_workout_distance VARCHAR(50) NOT NULL DEFAULT '',
    hk_workout_avg_heart_rate VARCHAR(50) NOT NULL DEFAULT '',
    hk_workout_active_energy_burned VARCHAR(50) NOT NULL DEFAULT ''
  );

  CREATE INDEX "index_hk_workout_id_on_workout_data"
  on workout_data using btree(hk_workout_id);
