use crate::db::models::user::User;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// programs (id) {
//     id -> Uuid,
//     created_at -> Timestamptz,
//     owner_id -> Nullable<Uuid>,
//     #[max_length = 50]
//     name -> Varchar,
//     #[max_length = 255]
//     description -> Varchar,
//     #[max_length = 50]
//     duration -> Varchar,
//     #[max_length = 255]
//     focus_areas -> Varchar,
//     #[max_length = 50]
//     target_audience -> Varchar,
//     #[max_length = 255]
//     program_image -> Varchar,
//     intensity -> IntensityChoices,
//     #[max_length = 255]
//     slug -> Varchar,
//     template -> Bool,
//     client_id -> Nullable<Uuid>,
//     active -> Bool,
// }

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::programs, check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User, foreign_key=owner_id))]
pub struct Program {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Relationships
    pub owner_id: Option<uuid::Uuid>,

    // Fields
    pub name: String,
    pub description: String,
    pub duration: String,
    pub focus_areas: String,
    pub target_audience: String,
    pub program_image: String,
    pub intensity: super::IntensityChoices,
    pub slug: String,
    pub template: bool,
    pub client_id: Option<uuid::Uuid>,
    pub active: bool,
    pub complete: bool,
}

#[derive(Insertable, Deserialize, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::programs)]
pub struct PatchProgram {
    pub name: Option<String>,
    pub description: Option<String>,
    pub duration: Option<String>,
    pub focus_areas: Option<String>,
    pub target_audience: Option<String>,
    pub program_image: Option<String>,
    pub intensity: Option<super::IntensityChoices>,
    pub slug: Option<String>,
    pub template: Option<bool>,
    pub client_id: Option<uuid::Uuid>,
    pub active: Option<bool>,
    pub complete: Option<bool>,
}

#[derive(Insertable, Deserialize, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::programs)]
pub struct NewProgram {
    pub name: String,
    pub description: String,
    // pub duration: String,
    pub focus_areas: String,
    pub target_audience: String,
    // pub program_image: String,
    pub intensity: super::IntensityChoices,
    // pub slug: String,
    pub template: bool,
    pub client_id: Option<uuid::Uuid>,
    pub active: bool,
}
