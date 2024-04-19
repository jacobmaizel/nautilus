use crate::db::models::client::Client;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// diesel::table! {
//     client_forms (id) {
//         id -> Uuid,
//         created_at -> Timestamptz,
//         client_id -> Nullable<Uuid>,
//         #[max_length = 500]
//         health_history -> Varchar,
//         #[max_length = 500]
//         lifestyle -> Varchar,
//         #[max_length = 500]
//         time_availability -> Varchar,
//         #[max_length = 500]
//         motivation -> Varchar,
//         #[max_length = 500]
//         preferences -> Varchar,
//         #[max_length = 500]
//         extra_details -> Varchar,
//     }
// }

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::client_forms, check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Client, foreign_key=client_id))]
pub struct ClientForm {
    // Meta
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Relationships
    pub client_id: Option<uuid::Uuid>,

    // Fields
    pub health_history: String,
    pub lifestyle: String,
    pub time_availability: String,
    pub motivation: String,
    pub preferences: String,
    pub extra_details: String,
}

#[derive(Insertable, Deserialize, Clone, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::client_forms)]
pub struct NewClientForm {
    // Relationships
    // pub client_id: uuid::Uuid,

    // Fields
    pub health_history: String,
    pub lifestyle: String,
    pub time_availability: String,
    pub motivation: String,
    pub preferences: String,
    pub extra_details: String,
}
