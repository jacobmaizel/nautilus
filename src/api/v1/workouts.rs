use crate::{
    db::models::{
        exercise::Exercise,
        workout::{NewWorkout, Workout, WorkoutWithExercises},
    },
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

pub fn workout_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_workouts).post(create_workout))
        .route(
            "/:workout_id",
            get(get_workout).put(update_workout).delete(delete_workout),
        )
}

async fn get_workout(
    State(state): State<Arc<AppState>>,
    Path(workout_path): Path<String>,
) -> AppResult<Json<WorkoutWithExercises>> {
    use crate::schema::workouts::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let mut base = workouts.into_boxed();

    base = match uuid::Uuid::from_str(&workout_path) {
        Ok(id_filter) => base.filter(id.eq(id_filter)),
        Err(_e) => base.filter(slug.eq(workout_path)),
    };

    let db_workout = base
        .select(Workout::as_select())
        .first::<Workout>(&mut conn)?;

    let exercises_belonging_to_workouts = Exercise::belonging_to(&db_workout)
        .select(Exercise::as_select())
        .load::<Exercise>(&mut conn)?;

    let res = WorkoutWithExercises {
        workout: db_workout,
        exercises: exercises_belonging_to_workouts,
    };

    Ok(Json(res))
}

async fn list_workouts(
    State(state): State<Arc<AppState>>,
    hm: QueryHmExt,
) -> AppResult<Json<Vec<WorkoutWithExercises>>> {
    use crate::schema::workouts::dsl::*;
    let mut base_q = workouts.into_boxed();

    let program_query =
        hm.0.get("program")
            .map(|val| val.as_str())
            .map(uuid::Uuid::from_str);

    let owner =
        hm.0.get("owner")
            .map(|val| val.as_str())
            .map(uuid::Uuid::from_str);

    let only_templates = hm.0.get("template").map(|val| val.as_str());

    if let Some(only_templates) = only_templates {
        match only_templates {
            "true" => {
                base_q = base_q.filter(template.eq(true));
            }

            "false" => {
                base_q = base_q.filter(template.eq(false));
            }
            _ => {}
        }
    };

    base_q = match owner {
        Some(Ok(val)) => base_q.filter(owner_id.eq(val)),
        Some(Err(_e)) => {
            return Err(custom(StatusCode::BAD_REQUEST, "Invalid Owner ID."));
        }
        None => base_q,
    };

    base_q = match program_query {
        Some(Ok(val)) => base_q.filter(program_id.eq(val)),
        Some(Err(_e)) => {
            return Err(custom(StatusCode::BAD_REQUEST, "Invalid Program ID."));
        }
        None => base_q,
    };

    let mut conn = state.db_pool.get_conn();
    let workouts_res = base_q
        .select(Workout::as_select())
        .load::<Workout>(&mut conn)?;

    let exercises_res: Vec<Exercise> = Exercise::belonging_to(&workouts_res)
        .select(Exercise::as_select())
        .load(&mut conn)?;

    let workout_with_exercises = exercises_res
        .grouped_by(&workouts_res)
        .into_iter()
        .zip(workouts_res)
        .map(|(exer, wk)| WorkoutWithExercises {
            workout: wk,
            exercises: exer,
        })
        .collect::<Vec<WorkoutWithExercises>>();

    // println!("output\n {:?}", workout_with_exercises);

    Ok(Json(workout_with_exercises))
}

async fn create_workout(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id_extracted): UserIdExtractor,
    JsonExtractor(body): JsonExtractor<NewWorkout>,
) -> AppResult<Json<WorkoutWithExercises>> {
    use crate::schema::workouts::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let mut new_slug = format_slug(body.name.clone());

    // check for slug uniqueness
    let slug_count: i64 = workouts
        .filter(slug.eq(new_slug.clone()))
        .count()
        .get_result(&mut conn)?;

    if slug_count > 0 {
        new_slug = format!("{}-{}", new_slug, &rand::random::<u32>().to_string());
    }

    let res = insert_into(workouts)
        .values((&body, owner_id.eq(user_id_extracted), slug.eq(new_slug)))
        .returning(Workout::as_returning())
        .get_result::<Workout>(&mut conn)?;

    let exercises_belonging_to_workouts = Exercise::belonging_to(&res)
        .select(Exercise::as_select())
        .load::<Exercise>(&mut conn)?;

    let wrk_exer = WorkoutWithExercises {
        workout: res,
        exercises: exercises_belonging_to_workouts,
    };

    Ok(Json(wrk_exer))
}

async fn update_workout(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id_extracted): UserIdExtractor,
    Path(workout_id): Path<uuid::Uuid>,
    JsonExtractor(body): JsonExtractor<NewWorkout>,
) -> AppResult<Json<WorkoutWithExercises>> {
    use crate::schema::workouts::dsl::*;

    let mut conn = state.db_pool.get_conn();
    let res = update(workouts)
        .filter(owner_id.eq(user_id_extracted).and(id.eq(workout_id)))
        .set(body)
        .returning(Workout::as_returning())
        .get_result::<Workout>(&mut conn)?;

    let exercises_belonging_to_workouts = Exercise::belonging_to(&res)
        .select(Exercise::as_select())
        .load::<Exercise>(&mut conn)?;

    let wrk_exer = WorkoutWithExercises {
        workout: res,
        exercises: exercises_belonging_to_workouts,
    };

    Ok(Json(wrk_exer))
}

async fn delete_workout(
    State(state): State<Arc<AppState>>,
    Path(workout_id_to_delete): Path<uuid::Uuid>,
    UserIdExtractor(user_id_ext): UserIdExtractor,
) -> AppResult<Json<usize>> {
    use crate::schema::workouts::dsl::*;

    let res = diesel::delete(workouts)
        .filter(id.eq(workout_id_to_delete).and(owner_id.eq(user_id_ext)))
        .execute(&mut state.db_pool.get_conn())?;

    Ok(Json(res))
}
