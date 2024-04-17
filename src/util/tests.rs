use crate::{
    api::{
        common::{api_fallback, healthcheck},
        v1_routes,
    },
    db::{
        models::user::{NewUser, User},
        DbConnection, MIGRATIONS,
    },
    server::AppState,
};
use axum::{routing::get, Extension, Router};
use diesel::{
    insert_into,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    Connection, PgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use std::sync::Arc;

pub struct TestContext {
    pub state: Arc<AppState>,
    pub router: Router,
    pub user: User,
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    pub fn new() -> Self {
        let state: Arc<AppState> = Arc::new(AppState::new());

        let user = create_test_user(&mut state.db_pool.test_conn());

        let router = build_test_router(state.clone(), user.clone());

        TestContext {
            state,
            router,
            user,
        }
    }
}

fn build_test_router(state: Arc<AppState>, user: User) -> Router {
    axum::Router::new()
        .nest("/v1", v1_routes())
        .route("/", get(healthcheck))
        // adding extension manually into the request to be used later in the handler
        // User ID Extractors
        .layer(Extension(user.id))
        .with_state(Arc::clone(&state))
}

////////////////////////////// figure this out
// static START: Once = Once::new();

pub fn create_test_user(conn: &mut DbConnection) -> User {
    let nu: NewUser = NewUser {
        user_type: crate::db::models::user::UserType::User,
        is_admin: true,
        onboarding_completed: true,
        first_name: "Test".to_string(),
        last_name: "Testlast".to_string(),
        user_name: "testuser".to_string(),
        email: "testuser@gmail.com".to_string(),
        phone_number: "123456789".to_string(),
        provider_id: "testprovider123".to_string(),
        image: "".to_string(),
        birthday: None,
        goals: "Goals are great!".to_string(),
        weight: 140,
        training_approach: "Train hard".to_string(),
        training_years: 12,
        training_specializations: "Strength, Cardio".to_string(),
        bio: "A great user bio!".to_string(),
        gender: "Male".to_string(),
        beta_access: true,
    };

    use crate::schema::users::dsl::*;

    insert_into(users)
        .values(&nu)
        .returning(User::as_returning())
        .get_result(conn)
        .unwrap()
}
