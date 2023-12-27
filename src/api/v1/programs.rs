use crate::{
    db::models::{
        exercise::Exercise,
        program::{NewProgram, Program},
        workout::{Workout, WorkoutWithExercises},
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
use diesel::{insert_into, prelude::*};
use http::StatusCode;
use serde::Serialize;
use std::{str::FromStr, sync::Arc};

pub fn program_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_programs).post(create_program))
        .route(
            "/:program_id",
            get(get_program).put(update_program).delete(delete_program),
        )
}

// #[debug_handler]
async fn get_program(
    State(state): State<Arc<AppState>>,
    Path(program_path): Path<String>,
) -> AppResult<Json<ProgramWithWorkouts>> {
    use crate::schema::programs::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let mut base = programs.into_boxed();

    base = match uuid::Uuid::from_str(&program_path) {
        Ok(id_filter) => base.filter(id.eq(id_filter)),
        Err(_e) => base.filter(slug.eq(program_path)),
    };

    let program_db_res = base
        .select(Program::as_select())
        .first::<Program>(&mut conn)?;

    let workouts_belonging_to_programs = Workout::belonging_to(&program_db_res)
        .select(Workout::as_select())
        .load::<Workout>(&mut conn)?;

    let exercises_belonging_to_workouts = Exercise::belonging_to(&workouts_belonging_to_programs)
        .select(Exercise::as_select())
        .load::<Exercise>(&mut conn)?;

    let grouped_exercises: Vec<Vec<Exercise>> =
        exercises_belonging_to_workouts.grouped_by(&workouts_belonging_to_programs);

    let workouts_with_exercises = workouts_belonging_to_programs
        .into_iter()
        .zip(grouped_exercises)
        .map(|(workout, exercises)| WorkoutWithExercises { workout, exercises })
        .collect();

    let res: ProgramWithWorkouts = ProgramWithWorkouts {
        program: program_db_res,
        workouts: workouts_with_exercises,
    };

    Ok(Json(res))
}

// #[debug_handler]
async fn list_programs(
    State(state): State<Arc<AppState>>,
    hm: QueryHmExt,
) -> AppResult<Json<Vec<ProgramWithWorkouts>>> {
    use crate::schema::programs::dsl as programs_dsl;
    let mut base_q = programs_dsl::programs.into_boxed();

    let owner_query =
        hm.0.get("owner")
            .map(|val| val.as_str())
            .map(uuid::Uuid::from_str);

    base_q = match owner_query {
        Some(Ok(val)) => base_q.filter(programs_dsl::owner_id.eq(val)),
        Some(Err(e)) => {
            return Err(custom(StatusCode::BAD_REQUEST, e.to_string()));
        }
        None => base_q,
    };

    let mut conn = state.db_pool.get_conn();

    let program_db_res = base_q
        .select(Program::as_select())
        .load::<Program>(&mut conn)?;

    let workouts_belonging_to_programs = Workout::belonging_to(&program_db_res)
        .select(Workout::as_select())
        .load::<Workout>(&mut conn)?;

    let exercises_belonging_to_workouts = Exercise::belonging_to(&workouts_belonging_to_programs)
        .select(Exercise::as_select())
        .load::<Exercise>(&mut conn)?;

    let grouped_exercises: Vec<Vec<Exercise>> =
        exercises_belonging_to_workouts.grouped_by(&workouts_belonging_to_programs);

    let workouts_with_exercises: Vec<Vec<(Workout, Vec<Exercise>)>> =
        workouts_belonging_to_programs
            .into_iter()
            .zip(grouped_exercises)
            .grouped_by(&program_db_res);

    let res: Vec<ProgramWithWorkouts> = program_db_res
        .into_iter()
        .zip(workouts_with_exercises)
        .map(ProgramWithWorkouts::from)
        .collect();

    Ok(Json(res))
}

#[derive(Serialize)]
pub struct ProgramWithWorkouts {
    #[serde(flatten)]
    pub program: Program,
    pub workouts: Vec<WorkoutWithExercises>,
}

impl From<(Program, Vec<(Workout, Vec<Exercise>)>)> for ProgramWithWorkouts {
    fn from(value: (Program, Vec<(Workout, Vec<Exercise>)>)) -> Self {
        let (program, workouts_with_exercises) = value;

        let workouts_with_exercises = workouts_with_exercises
            .into_iter()
            .map(|(workout, exercises)| WorkoutWithExercises { workout, exercises })
            .collect();

        ProgramWithWorkouts {
            program,
            workouts: workouts_with_exercises,
        }
    }
}

// #[debug_handler]
async fn create_program(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id_ext): UserIdExtractor,
    JsonExtractor(body): JsonExtractor<NewProgram>,
) -> AppResult<Json<Program>> {
    use crate::schema::programs::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let mut new_slug = format_slug(body.name.clone());

    // check for slug uniqueness
    let slug_count: i64 = programs
        .filter(slug.eq(new_slug.clone()))
        .count()
        .get_result(&mut conn)?;

    if slug_count > 0 {
        new_slug = format!("{}-{}", new_slug, &rand::random::<u32>().to_string());
    }

    let res = insert_into(programs)
        .values((&body, owner_id.eq(user_id_ext), slug.eq(new_slug)))
        .returning(Program::as_returning())
        .get_result::<Program>(&mut state.db_pool.get_conn())?;

    Ok(Json(res))
}

// #[debug_handler]
async fn update_program(
    State(state): State<Arc<AppState>>,
    Path(program_id_to_update): Path<uuid::Uuid>,
    JsonExtractor(body): JsonExtractor<NewProgram>,
) -> AppResult<Json<Program>> {
    use crate::schema::programs::dsl::*;

    // println!("Updating program with id: {:?}", program_id_to_update);

    let res = diesel::update(programs)
        .filter(id.eq(program_id_to_update))
        .set(body)
        .returning(Program::as_select())
        .get_result::<Program>(&mut state.db_pool.get_conn())?;

    Ok(Json(res))
}

// #[debug_handler]
async fn delete_program(
    State(state): State<Arc<AppState>>,
    Path(program_id_to_update): Path<uuid::Uuid>,
    UserIdExtractor(user_id_ext): UserIdExtractor,
) -> AppResult<Json<usize>> {
    use crate::schema::programs::dsl::*;

    let res = diesel::delete(programs)
        .filter(id.eq(program_id_to_update).and(owner_id.eq(user_id_ext)))
        .execute(&mut state.db_pool.get_conn())?;

    Ok(Json(res))
}
