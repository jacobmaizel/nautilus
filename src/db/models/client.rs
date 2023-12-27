use super::user::PublicUser;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(diesel_derive_enum::DbEnum, Debug, Serialize, Clone, Deserialize, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::InviteStates"]
pub enum InviteStates {
    Pending,
    Rejected,
    Accepted,
}

// clients (id) {
//     id -> Uuid,
//     created_at -> Timestamptz,
//     user_id -> Nullable<Uuid>,
//     trainer_id -> Nullable<Uuid>,
//     is_active -> Bool,
//     #[max_length = 50]
//     status -> Varchar,
//     invite -> InviteStates,
// }
//
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Identifiable)]
#[diesel(table_name = crate::schema::clients, check_for_backend(diesel::pg::Pg))]
// #[diesel(belongs_to(User, foreign_key=trainer_id))]
// #[diesel(belongs_to(User))]
pub struct Client {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Relationships
    pub user_id: Option<uuid::Uuid>,
    pub trainer_id: Option<uuid::Uuid>,

    // Fields
    pub is_active: bool,
    pub status: String,
    pub invite: InviteStates,
}

#[derive(Serialize)]
pub struct ClientWithUser {
    #[serde(flatten)]
    pub client: Client,
    pub user: PublicUser,
}

#[derive(Insertable, Deserialize, Clone, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::clients)]
pub struct NewClient {
    // pub id: uuid::Uuid,
    // pub created_at: chrono::NaiveDateTime,

    // Relationships
    pub user_id: uuid::Uuid,
    // pub trainer_id: Option<uuid::Uuid>,

    // Fields
    pub is_active: bool,
    pub status: String,
    pub invite: InviteStates,
}

#[derive(Insertable, Deserialize, Clone, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::clients)]
pub struct PatchClient {
    pub is_active: Option<bool>,
    pub status: Option<String>,
    pub invite: Option<InviteStates>,
}
