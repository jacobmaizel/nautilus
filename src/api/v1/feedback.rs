use crate::{
    db::models::feedback::{Feedback, NewFeedback},
    server::AppState,
    types::AppResult,
    util::extractors::{JsonExtractor, UserIdExtractor},
};
use axum::{extract::State, routing::*, Json};
use diesel::{insert_into, prelude::*};
use std::sync::Arc;

pub fn feedback_routes() -> Router<Arc<AppState>> {
    Router::new().route("/", post(create_feedback))
}

pub async fn create_feedback(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(u_id): UserIdExtractor,
    JsonExtractor(body): JsonExtractor<NewFeedback>,
) -> AppResult<Json<Feedback>> {
    use crate::schema::feedback::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let res: Feedback = insert_into(feedback)
        .values((&body, user_id.eq(u_id)))
        .returning(Feedback::as_returning())
        .get_result(&mut conn)?;

    Ok(Json(res))
}
