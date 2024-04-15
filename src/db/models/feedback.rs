use crate::db::models::user::User;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// id -> Uuid,
// created_at -> Timestamptz,
// user_id -> Nullable<Uuid>,
// #[max_length = 100]
// title -> Varchar,
// #[max_length = 500]
// description -> Varchar,
// followup -> Bool,
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::feedback, check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User, foreign_key=user_id))]
pub struct Feedback {
    // Meta
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Relationships
    pub user_id: Option<uuid::Uuid>,

    // Fields
    pub title: String,
    pub description: String,
    pub followup: bool,
}

#[derive(Insertable, Deserialize, Clone, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::feedback)]
pub struct NewFeedback {
    // Fields
    pub title: String,
    pub description: String,
    pub followup: bool,
}
