use crate::{
    db::models::{
        exercise::Exercise,
        program::Program,
        workout::{NewWorkout, PatchWorkout, Workout, WorkoutWithExercises},
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
use diesel::{insert_into, prelude::*, update};
use http::StatusCode;
use std::{str::FromStr, sync::Arc};

pub fn workout_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_workouts).post(create_workout))
        .route(
            "/:workout_id",
            get(get_workout)
                .patch(update_workout)
                .delete(delete_workout),
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
        .order(sequence.asc())
        .select(Workout::as_select())
        .load::<Workout>(&mut conn)?;

    use crate::schema::exercises::dsl as exer_dsl;
    let exercises_res: Vec<Exercise> = Exercise::belonging_to(&workouts_res)
        .order(exer_dsl::sequence.asc())
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

#[derive(Queryable, Clone)]
struct ProgActiveCounter {
    id: uuid::Uuid,
    complete: bool,
}

async fn update_workout(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(req_user_id): UserIdExtractor,
    Path(workout_id): Path<uuid::Uuid>,
    JsonExtractor(body): JsonExtractor<PatchWorkout>,
) -> AppResult<Json<WorkoutWithExercises>> {
    use crate::schema::{clients::dsl as cli_dsl, programs::dsl as pro_dsl, workouts::dsl::*};

    let mut conn = state.db_pool.get_conn();

    // only the owner of the workout or a client that is assigned a program in which this
    // workout is connected to can update this workout.

    let req_user_client: Result<uuid::Uuid, diesel::result::Error> = cli_dsl::clients
        .filter(cli_dsl::user_id.eq(req_user_id))
        .select(cli_dsl::id)
        .first::<uuid::Uuid>(&mut conn);

    let workout_from_path: Workout = workouts
        .filter(id.eq(workout_id))
        .select(Workout::as_select())
        .first(&mut conn)?;

    let program_for_workout: Result<Program, diesel::result::Error> = pro_dsl::programs
        .filter(pro_dsl::id.nullable().eq(workout_from_path.program_id))
        .select(Program::as_select())
        .first(&mut conn);

    let req_user_is_workout_owner = workout_from_path.owner_id == Some(req_user_id);
    let req_user_is_client_from_workout_program = req_user_client.is_ok_and(|req_client_id| {
        program_for_workout
            .is_ok_and(|prog| prog.client_id.is_some_and(|cli_id| cli_id == req_client_id))
    });

    if req_user_is_workout_owner || req_user_is_client_from_workout_program {
        let workout_filter = id.eq(workout_id);

        let res = update(workouts)
            .filter(workout_filter)
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

        // if all of the programs workouts are not complete, set program as complete!
        if let Some(prog_id) = workout_from_path.program_id {
            println!("inside this thing");
            let prog_workouts: Result<Vec<ProgActiveCounter>, diesel::result::Error> = workouts
                .filter(program_id.eq(prog_id))
                .select((id, complete))
                .load::<ProgActiveCounter>(&mut conn);

            if let Ok(wkts) = prog_workouts {
                println!("we had workouts");
                let completed = wkts.clone().into_iter().filter(|w| w.complete).count();

                println!(
                    "completed count {}, total count {}",
                    completed.clone(),
                    wkts.len()
                );

                if completed == wkts.len() {
                    // all of the workouts have been completed! update the program
                    let _res = update(pro_dsl::programs)
                        .filter(pro_dsl::id.eq(prog_id))
                        .set(pro_dsl::complete.eq(true))
                        .execute(&mut conn);
                }
            }
        };

        return Ok(Json(wrk_exer));
    }

    Err(unauthorized())

    // here we know atleast one of the following
    // 1. the req user is a client
    // 2. this workout is connected to a program
    // 3. the workout from path owner might be the owner
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
