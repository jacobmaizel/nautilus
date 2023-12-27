pub mod v1;
use self::v1::{
    beta::beta_routes, certifications::certification_routes, clients::client_routes,
    exercises::exercise_routes, notification::notification_routes, programs::program_routes,
    users::user_routes, workouts::workout_routes,
};
use crate::server::AppState;
use axum::Router;
use std::sync::Arc;

pub fn v1_routes() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/users", user_routes())
        .nest("/beta", beta_routes())
        .nest("/certifications", certification_routes())
        .nest("/programs", program_routes())
        .nest("/workouts", workout_routes())
        .nest("/exercises", exercise_routes())
        .nest("/clients", client_routes())
        .nest("/notifications", notification_routes())
}
