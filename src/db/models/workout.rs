use super::{exercise::Exercise, IntensityChoices};
#[allow(unused_imports)]
use crate::db::models::{program::Program, user::User};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::workouts, check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Program))]
#[diesel(belongs_to(User, foreign_key=owner_id))]
pub struct Workout {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Relationships
    pub program_id: Option<uuid::Uuid>,
    pub owner_id: Option<uuid::Uuid>,

    // Fields
    pub name: String,
    pub description: String,
    pub duration: String,
    pub sequence: i32,
    pub week: i32,
    pub intensity: IntensityChoices,
    pub workout_type: String,
    pub equipment_needed: String,
    pub image: String,
    pub video: String,
    pub template: bool,
    pub slug: String,
}

#[derive(Serialize)]
pub struct WorkoutWithExercises {
    #[serde(flatten)]
    pub workout: Workout,
    pub exercises: Vec<Exercise>,
}

#[derive(Insertable, Deserialize, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::workouts)]
pub struct NewWorkout {
    pub program_id: Option<uuid::Uuid>,
    pub name: String,
    pub description: String,
    pub duration: String,
    pub sequence: i32,
    pub week: i32,
    pub intensity: IntensityChoices,
    pub workout_type: String,
    pub equipment_needed: String,
    pub image: String,
    pub video: String,
    pub template: bool,
    // pub slug: String,
}
