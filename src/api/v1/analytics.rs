use crate::{
    server::AppState,
    types::{self, AppResult},
    util::extractors::{JsonExtractor, UserIdExtractor},
};
use axum::{extract::State, routing::*, Json};
use chrono::Utc;
use diesel::{
    dsl::exists,
    insert_into,
    pg::sql_types,
    prelude::*,
    select, sql_query,
    sql_types::{BigInt, Integer, Timestamptz},
    update,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn analytics_routes() -> Router<Arc<AppState>> {
    Router::new().route("/clients_by_month", get(clients_by_month))
}

pub async fn clients_by_month(
    UserIdExtractor(u_id): UserIdExtractor,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    use crate::schema::clients::dsl::*;
    /*
     * {
     *  "num_clients": 123,
     *  "month": <date>
     * }
     */
    let mut conn = state.db_pool.get_conn();

    let q = sql_query("SELECT date_trunc('month', accepted_invite_at) AS accepted_invite_month, count(id) as client_count FROM clients WHERE clients.trainer_id = $1 GROUP BY accepted_invite_month;");

    // let q = sql_query("SELECT date_trunc('month', accepted_invite_at) AS accepted_invite_month,
    // count(id) as client_count FROM clients WHERE user_id = ? GROUP BY accepted_invite_month");

    let res: Vec<ClientsByMonth> = q.bind::<sql_types::Uuid, _>(u_id).get_results(&mut conn)?;

    Ok(Json(serde_json::json!({"data": res})))
}

#[derive(Debug, Clone, QueryableByName, Serialize, Deserialize)]
pub struct ClientsByMonth {
    #[diesel(sql_type = Timestamptz)]
    pub accepted_invite_month: chrono::DateTime<Utc>,
    #[diesel(sql_type = BigInt)]
    pub client_count: i64,
}
