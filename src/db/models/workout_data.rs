use super::workout::Workout;
#[allow(unused_imports)]
use crate::db::models::{program::Program, user::User};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::workout_data, check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Workout))]
pub struct WorkoutData {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Relationships
    pub workout_id: Option<uuid::Uuid>,

    // Fields
    pub hk_workout_id: String,
    pub hk_location_type: String,
    pub hk_workout_activity_type: String,
    pub hk_workout_duration_secs: String,
    pub hk_workout_start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub hk_workout_end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub hk_workout_distance: String,
    pub hk_workout_avg_heart_rate: String,
    pub hk_workout_active_energy_burned: String,
}

#[derive(Insertable, Deserialize, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::workout_data)]
pub struct NewWorkoutData {
    // pub workout_id: Option<uuid::Uuid>,
    pub hk_workout_id: String,
    pub hk_location_type: String,
    pub hk_workout_activity_type: String,
    pub hk_workout_duration_secs: String,
    pub hk_workout_start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub hk_workout_end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub hk_workout_distance: String,
    pub hk_workout_avg_heart_rate: String,
    pub hk_workout_active_energy_burned: String,
}
