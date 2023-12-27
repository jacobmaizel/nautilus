use crate::{
    db::{
        models::betacode::{BetaCode, NewBetaCode},
        users::{get_user, guard_admin},
    },
    error::{api_error, json_msg},
    server::AppState,
    types::{self, AppResult},
    util::extractors::{JsonExtractor, UserIdExtractor},
};
use axum::{extract::State, routing::*, Json};
use diesel::{dsl::exists, insert_into, prelude::*, select, update};
use http::StatusCode;
use std::sync::Arc;

pub fn beta_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/validate", post(validate_beta_code))
        .route("/", post(create_beta_code).delete(remove_beta_key))
        .route("/resetaccess", post(resetaccess))
}

pub async fn validate_beta_code(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id): UserIdExtractor,
    JsonExtractor(body): JsonExtractor<NewBetaCode>,
) -> types::DBResult<serde_json::Value> {
    use crate::schema::betacode::dsl::*;

    let conn = &mut state.db_pool.get_conn();

    let res: bool = select(exists(betacode.select(id).filter(code.eq(body.code))))
        .get_result(conn)
        .map_err(api_error)?;

    // let ok: bool = select(exists(betacode.select(id).filter(code.eq(body.code))))
    //     .get_result(conn)
    //     .map_err(api_error)?;

    if res {
        use crate::schema::users::dsl::*;

        let _ = update(users)
            .set(beta_access.eq(true))
            .filter(id.eq(user_id))
            .execute(&mut state.db_pool.get_conn())
            .map_err(api_error)?;

        return Ok(json_msg("Valid"));
    }
    Err((StatusCode::BAD_REQUEST, json_msg("Invalid")))
}

pub async fn create_beta_code(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id): UserIdExtractor,
    JsonExtractor(body): JsonExtractor<NewBetaCode>,
) -> types::DBResult<BetaCode> {
    /* Restricted to only admin users */
    use crate::schema::betacode::dsl::*;

    let conn = &mut state.db_pool.get_conn();

    let user = get_user(user_id, conn).map_err(api_error)?;

    //     user.phone_number.clone

    if !user.is_admin {
        return Err((StatusCode::UNAUTHORIZED, json_msg("Unauthorized")));
    }

    let bc = insert_into(betacode)
        .values(&body)
        .returning(BetaCode::as_returning())
        .get_result(conn)
        .map_err(api_error)?;

    Ok(Json(bc))
}

pub async fn remove_beta_key(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id): UserIdExtractor,
    JsonExtractor(body): JsonExtractor<NewBetaCode>,
) -> AppResult<Json<serde_json::Value>> {
    use crate::schema::betacode::dsl::*;

    let mut conn = state.db_pool.get_conn();
    guard_admin(user_id, &mut conn)?;

    let rows: usize = diesel::delete(betacode.filter(code.eq(body.code)))
        .execute(&mut state.db_pool.get_conn())?;

    Ok(json_msg(rows.to_string().as_str()))
}

pub async fn resetaccess(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id): UserIdExtractor,
) -> types::DBResult<serde_json::Value> {
    use crate::schema::users::dsl::*;

    let conn = &mut state.db_pool.get_conn();
    let user = get_user(user_id, conn).map_err(api_error)?;

    if !user.is_admin {
        return Err((StatusCode::UNAUTHORIZED, json_msg("Unauthorized")));
    }

    let _ = update(users)
        .set(beta_access.eq(false))
        .filter(id.eq(user_id))
        .execute(&mut state.db_pool.get_conn())
        .map_err(api_error)?;

    Ok(json_msg("Valid"))
}
