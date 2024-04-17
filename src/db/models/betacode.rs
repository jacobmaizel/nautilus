use diesel::prelude::*;
use serde::Serialize;
use serde_derive::Deserialize;

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::betacode, check_for_backend(diesel::pg::Pg))]
pub struct BetaCode {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,

    pub code: String,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::betacode)]
pub struct NewBetaCode {
    pub code: String,
}
