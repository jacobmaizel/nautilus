use super::IntensityChoices;
use crate::db::models::{user::User, workout::Workout};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::exercises, check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Workout))]
#[diesel(belongs_to(User, foreign_key=owner_id))]
pub struct Exercise {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Relationships
    pub workout_id: Option<uuid::Uuid>,
    pub owner_id: Option<uuid::Uuid>,

    // Fields
    pub name: String,
    pub description: String,
    // Use duration for time-based exercises, reps for rep-based exercises
    pub duration: String,
    pub reps: String,
    pub sets: i32,
    pub rest_period: String,
    pub intensity: IntensityChoices,
    pub equipment: String,
    pub muscle_groups: String,
    pub image: String,
    pub video: String,
    pub instructions: String,
    pub sequence: i32,
    pub slug: String,
}

#[derive(Insertable, Deserialize, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::exercises)]
pub struct NewExercise {
    pub workout_id: Option<uuid::Uuid>,
    pub name: String,
    pub description: String,
    pub duration: String,
    pub reps: String,
    pub sets: i32,
    pub rest_period: String,
    pub intensity: IntensityChoices,
    pub equipment: String,
    pub muscle_groups: String,
    pub image: String,
    pub video: String,
    pub instructions: String,
    pub sequence: i32,
    // pub slug: String,
}
