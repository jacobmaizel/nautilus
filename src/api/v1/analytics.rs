use crate::{server::AppState, types::AppResult, util::extractors::UserIdExtractor};
use axum::{extract::State, routing::*, Json};
use diesel::{
    pg::sql_types,
    prelude::*,
    sql_query,
    sql_types::{BigInt, Text},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn analytics_routes() -> Router<Arc<AppState>> {
    Router::new().route("/clients_by_month", get(clients_by_month))
}

pub async fn clients_by_month(
    UserIdExtractor(u_id): UserIdExtractor,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let mut conn = state.db_pool.get_conn();

    let q = sql_query("SELECT to_char(date_trunc('month', accepted_invite_at), 'Mon') AS accepted_invite_month, count(id) as client_count FROM clients WHERE clients.trainer_id = $1 GROUP BY accepted_invite_month;");

    let res: Vec<ClientsByMonth> = q.bind::<sql_types::Uuid, _>(u_id).get_results(&mut conn)?;

    Ok(Json(serde_json::json!({"data": res})))
}

#[derive(Debug, Clone, QueryableByName, Serialize, Deserialize)]
pub struct ClientsByMonth {
    #[diesel(sql_type = Text)]
    pub accepted_invite_month: String,
    #[diesel(sql_type = BigInt)]
    pub client_count: i64,
}
