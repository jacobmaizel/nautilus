use crate::{
    db::{models::user::User, DbConnection},
    types::AppResult,
};
use diesel::{insert_into, prelude::*};
use serde::{Deserialize, Serialize};

// diesel::table! {
//     notifications (id) {
//         id -> Uuid,
//         created_at -> Timestamptz,
//         sender_id -> Nullable<Uuid>,
//         user_id -> Nullable<Uuid>,
//         #[max_length = 50]
//         title -> Varchar,
//         #[max_length = 255]
//         content -> Varchar,
//         #[max_length = 50]
//         category -> Varchar,
//         #[max_length = 50]
//         status -> Varchar,
//         opened_at -> Nullable<Timestamptz>,
//         data -> Nullable<Jsonb>,
//     }
// }

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Identifiable, Associations)]
#[diesel(table_name = crate::schema::notifications, check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User, foreign_key=user_id))]
pub struct Notification {
    // Meta
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Relationships
    pub user_id: Option<uuid::Uuid>,
    pub sender_id: Option<uuid::Uuid>,

    // Fields
    pub title: String,
    pub content: String,
    pub category: String,
    pub status: String,
    pub opened_at: Option<chrono::DateTime<chrono::Utc>>,
    pub data: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize, Clone, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::notifications)]
pub struct NewNotification {
    // Relationships
    pub user_id: uuid::Uuid,
    pub sender_id: uuid::Uuid,

    // Fields
    pub title: String,
    pub content: String,
    pub category: String,
    pub status: String,
    pub opened_at: Option<chrono::DateTime<chrono::Utc>>,
    pub data: Option<serde_json::Value>,
}

impl NewNotification {
    pub fn new(
        from_user: uuid::Uuid,
        to_user: uuid::Uuid,
        title: String,
        content: String,
        category: String,
        status: String,
        data: Option<serde_json::Value>,
    ) -> NewNotification {
        NewNotification {
            user_id: to_user,
            sender_id: from_user,
            title,
            content,
            category,
            status,
            data,
            opened_at: None,
        }
    }

    pub fn send(&self, conn: &mut DbConnection) -> AppResult<Notification> {
        use crate::schema::notifications::dsl::*;

        let db_res = insert_into(notifications)
            .values(self)
            .returning(Notification::as_returning())
            .get_result(conn)?;

        Ok(db_res)
    }
}
