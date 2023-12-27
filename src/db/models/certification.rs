#[allow(unused_imports)]
use crate::db::models::user::User;
use diesel::prelude::*;
use serde::Serialize;
use serde_derive::Deserialize;

// id -> Uuid,
// created_at -> Timestamptz,
// user_id -> Nullable<Uuid>,
// #[max_length = 50]
// name -> Varchar,
// expiration -> Nullable<Date>,

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::certifications, check_for_backend(diesel::pg::Pg))]
pub struct Certification {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<uuid::Uuid>,
    pub name: String,
    pub expiration: Option<chrono::NaiveDate>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = crate::schema::certifications)]
pub struct NewCertification {
    pub name: String,
    pub user_id: Option<uuid::Uuid>,
    pub expiration: Option<chrono::NaiveDate>,
}
