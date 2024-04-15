use crate::{
    db::models::certification::{Certification, NewCertification},
    error::api_error,
    server::AppState,
    types::{self, AppResult},
    util::extractors::{JsonExtractor, QueryHmExt, UserIdExtractor},
};
use axum::{extract::State, routing::get, Json};
use diesel::{insert_into, prelude::*};
use std::sync::Arc;

pub fn certification_routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route(
        "/",
        get(get_certifications)
            .post(create_certification)
            .delete(delete_certifications),
    )
}

async fn get_certifications(
    hm: QueryHmExt,
    State(state): State<Arc<AppState>>,
) -> types::DBResult<Vec<Certification>> {
    use crate::schema::{certifications::dsl as cert_dsl, users::dsl::*};

    let mut exp = cert_dsl::certifications.into_boxed();

    let name_query_param = hm.0.get("user").map(|val| val.as_str());

    exp = match name_query_param {
        Some(val) => {
            let db_user_id = users
                .select(id)
                .filter(user_name.eq(val))
                .first::<uuid::Uuid>(&mut state.db_pool.get_conn())
                .map_err(api_error)?;

            exp.filter(cert_dsl::user_id.eq(Some(db_user_id)))
        }
        None => exp,
    };

    let res = exp
        .load::<Certification>(&mut state.db_pool.get_conn())
        .map_err(api_error)?;

    Ok(Json(res))
}

async fn create_certification(
    UserIdExtractor(u_id): UserIdExtractor,
    State(state): State<Arc<AppState>>,
    JsonExtractor(body): JsonExtractor<NewCertification>,
) -> types::DBResult<Certification> {
    use crate::schema::certifications::dsl::*;

    // NOTE: This needs to be properly handled on frontend, but for now
    // I am just going to delete any name dupes so I can do a simple POST to
    // handle ALL changes in cert list
    //
    let mut conn = state.db_pool.get_conn();

    let existing_name = certifications
        .filter(name.eq(body.name.clone()).and(user_id.eq(u_id)))
        .select(id)
        .first::<uuid::Uuid>(&mut conn);

    if let Ok(existing_cert_id) = existing_name {
        let _ = diesel::delete(certifications.filter(id.eq(existing_cert_id))).execute(&mut conn);
    }

    let val = NewCertification {
        user_id: Some(u_id),
        ..body
    };

    let cert = insert_into(certifications)
        .values(&val)
        .returning(Certification::as_returning())
        .get_result(&mut state.db_pool.get_conn())
        .map_err(api_error)?;

    Ok(Json(cert))
}

async fn delete_certifications(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(u_id): UserIdExtractor,
) -> AppResult<Json<serde_json::Value>> {
    use crate::schema::certifications::dsl::*;

    let mut conn = state.db_pool.get_conn();
    let res: usize = diesel::delete(certifications.filter(user_id.eq(u_id))).execute(&mut conn)?;

    Ok(Json(serde_json::json!({"deleted": res})))
}
