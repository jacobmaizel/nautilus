use crate::{
    db::models::{
        client::Client,
        exercise::Exercise,
        program::{NewProgram, PatchProgram, Program},
        workout::{Workout, WorkoutWithExercises},
    },
    error::{custom, unauthorized},
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
        .route("/active", get(get_active_program))
        .route(
            "/:program_id",
            get(get_program).put(update_program).delete(delete_program),
        )
}

async fn get_active_program(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(req_user_id): UserIdExtractor,
) -> AppResult<Json<ProgramWithWorkouts>> {
    use crate::schema::{clients::dsl as c_dsl, programs::dsl::*};

    let mut conn = state.db_pool.get_conn();

    let base = programs.into_boxed();

    // user -> client -> programs.assigned_to

    let client: Client = c_dsl::clients
        .filter(c_dsl::user_id.eq(req_user_id))
        .select(Client::as_select())
        .first(&mut conn)?;

    let program_db_res = base
        .filter(client_id.eq(client.id).and(active.eq(true)))
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

async fn get_program(
    State(state): State<Arc<AppState>>,
    Path(program_path): Path<String>,
) -> AppResult<Json<ProgramWithWorkouts>> {
    use crate::schema::programs::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let mut base = programs.into_boxed();

    base = match uuid::Uuid::from_str(&program_path) {
        Ok(id_filter) => base.filter(id.eq(id_filter)),
        Err(_) => base.filter(slug.eq(program_path)),
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

    if let Some(Ok(active_qp)) = hm.0.get("active").map(|val| bool::from_str(val)) {
        base_q = base_q.filter(programs_dsl::active.eq(active_qp))
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
    UserIdExtractor(req_user_id): UserIdExtractor,
    Path(program_id_to_update): Path<uuid::Uuid>,
    JsonExtractor(body): JsonExtractor<PatchProgram>,
) -> AppResult<Json<Program>> {
    use crate::schema::{programs::dsl::*, users::dsl as u_dsl};

    // println!("Updating program with id: {:?}", program_id_to_update);
    // req user needs to be owner of program

    let mut conn = state.db_pool.get_conn();

    let req_user_is_admin: bool = u_dsl::users
        .filter(u_dsl::id.eq(req_user_id))
        .select(u_dsl::is_admin)
        .first(&mut conn)?;

    let req_program_owner_id: Option<uuid::Uuid> = programs
        .filter(id.eq(program_id_to_update))
        .select(owner_id)
        .first(&mut conn)?;

    if req_program_owner_id == Some(req_user_id) || req_user_is_admin {
        let res = diesel::update(programs)
            .filter(id.eq(program_id_to_update))
            .set(body)
            .returning(Program::as_select())
            .get_result::<Program>(&mut state.db_pool.get_conn())?;

        return Ok(Json(res));
    }
    Err(unauthorized())
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
