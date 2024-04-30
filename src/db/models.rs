use serde::{Deserialize, Serialize};
pub mod betacode;
pub mod certification;
pub mod client;
pub mod client_form;
pub mod exercise;
pub mod feedback;
pub mod notification;
pub mod program;
pub mod user;
pub mod workout;
pub mod workout_data;

#[derive(diesel_derive_enum::DbEnum, Debug, Serialize, Clone, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::IntensityChoices"]
pub enum IntensityChoices {
    Low,
    Medium,
    High,
}
