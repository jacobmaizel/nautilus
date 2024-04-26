use crate::{
    db::models::exercise::{Exercise, NewExercise},
    error::custom,
    server::AppState,
    types::AppResult,
    util::{
        extractors::{JsonExtractor, Path, QueryHmExt, UserIdExtractor},
        format_slug,
    },
};
use axum::{extract::State, routing::get, Json, Router};
use diesel::{insert_into, prelude::*, update};
use http::StatusCode;
use std::{str::FromStr, sync::Arc};

pub fn exercise_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_exercises).post(create_exercise))
        .route(
            "/:exercise_id",
            get(get_exercise)
                .delete(delete_exercise)
                .put(update_exercise),
        )
}

async fn get_exercise(
    State(state): State<Arc<AppState>>,
    Path(exercise_id): Path<uuid::Uuid>,
) -> AppResult<Json<Exercise>> {
    use crate::schema::exercises::dsl::*;

    let res = exercises
        .select(Exercise::as_select())
        .filter(id.eq(exercise_id))
        .first::<Exercise>(&mut state.db_pool.get_conn())?;

    Ok(Json(res))
}

async fn list_exercises(
    State(state): State<Arc<AppState>>,
    hm: QueryHmExt,
) -> AppResult<Json<Vec<Exercise>>> {
    use crate::schema::exercises::dsl::*;
    let mut base_q = exercises.into_boxed();

    let workout_id_query =
        hm.0.get("workout")
            .map(|val| val.as_str())
            .map(uuid::Uuid::from_str);

    base_q = match workout_id_query {
        Some(Ok(val)) => base_q.filter(workout_id.eq(val)),
        Some(Err(e)) => {
            return Err(custom(StatusCode::BAD_REQUEST, e.to_string()));
        }
        None => base_q,
    };

    let res = base_q
        .order(sequence.asc())
        .select(Exercise::as_select())
        .load::<Exercise>(&mut state.db_pool.get_conn())?;

    Ok(Json(res))
}

async fn create_exercise(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(u_id_ext): UserIdExtractor,
    JsonExtractor(body): JsonExtractor<NewExercise>,
) -> AppResult<Json<Exercise>> {
    use crate::schema::exercises::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let mut new_slug = format_slug(body.name.clone());

    // check for slug uniqueness
    let slug_count: i64 = exercises
        .filter(slug.eq(new_slug.clone()))
        .count()
        .get_result(&mut conn)?;

    if slug_count > 0 {
        new_slug = format!("{}-{}", new_slug, &rand::random::<u32>().to_string());
    }

    let res = insert_into(exercises)
        .values((&body, owner_id.eq(u_id_ext), slug.eq(new_slug)))
        .returning(Exercise::as_returning())
        .get_result::<Exercise>(&mut conn)?;

    Ok(Json(res))
}

async fn delete_exercise(
    State(state): State<Arc<AppState>>,
    Path(exercise_id_to_delete): Path<uuid::Uuid>,
    UserIdExtractor(user_id_ext): UserIdExtractor,
) -> AppResult<Json<usize>> {
    use crate::schema::exercises::dsl::*;

    let res = diesel::delete(exercises)
        .filter(id.eq(exercise_id_to_delete).and(owner_id.eq(user_id_ext)))
        .execute(&mut state.db_pool.get_conn())?;

    Ok(Json(res))
}

async fn update_exercise(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id_extracted): UserIdExtractor,
    Path(exercise_id): Path<uuid::Uuid>,
    JsonExtractor(body): JsonExtractor<NewExercise>,
) -> AppResult<Json<Exercise>> {
    use crate::schema::exercises::dsl::*;

    let mut conn = state.db_pool.get_conn();
    let res = update(exercises)
        .filter(owner_id.eq(user_id_extracted).and(id.eq(exercise_id)))
        .set(body)
        .returning(Exercise::as_returning())
        .get_result::<Exercise>(&mut conn)?;

    Ok(Json(res))
}
