use crate::{
    db::models::{
        self,
        user::{NewUser, PublicUser, User},
    },
    error::{api_error, custom, json_msg},
    server::AppState,
    types::{self, AppResult, DBResult, JsonObject},
    util::extractors::{JsonExtractor, Path, QueryHmExt, UserIdExtractor},
};
use axum::{extract::State, routing::*, Json};
use diesel::{dsl::exists, prelude::*, select, update};
use http::StatusCode;
use std::sync::Arc;

pub fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:user_name_path", get(get_user_by_username))
        .route("/me", get(get_me).patch(update_me))
        .route("/exists", get(check_username_exists))
        .route("/", get(admin_list_users))
}

// #[instrument(skip(state))]
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id): UserIdExtractor,
) -> DBResult<User> {
    use crate::schema::users::dsl::*;

    let user = users
        .select(models::user::User::as_select())
        .filter(id.eq(user_id))
        .first(&mut state.db_pool.get_conn())
        .map_err(api_error)?;

    Ok(Json(user))
}

// #[instrument(skip(state))]
pub async fn update_me(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id): UserIdExtractor,
    JsonExtractor(body): JsonExtractor<NewUser>,
) -> types::DBResult<models::user::User> {
    use crate::schema::users::dsl::*;

    let res = update(users)
        .filter(id.eq(user_id))
        .set(body)
        .returning(User::as_select())
        .get_result::<User>(&mut state.db_pool.get_conn())
        .map_err(api_error)?;

    Ok(Json(res))
}
// #[instrument(skip(state))]
pub async fn get_user_by_username(
    State(state): State<Arc<AppState>>,
    Path(user_name_path): Path<String>,
) -> AppResult<Json<PublicUser>> {
    use crate::schema::users::dsl::*;

    let mut base = users.into_boxed();
    let name_or_id = uuid::Uuid::parse_str(&user_name_path);

    base = match name_or_id {
        Ok(u_id) => base.filter(id.eq(u_id)),
        Err(_) => base.filter(user_name.eq(user_name_path)),
    };

    let user = base
        .select(models::user::PublicUser::as_select())
        .first(&mut state.db_pool.get_conn())?;

    Ok(Json(user))
}

// #[instrument(skip(state))]
pub async fn check_username_exists(
    State(state): State<Arc<AppState>>,
    hm: QueryHmExt,
) -> (StatusCode, JsonObject) {
    use crate::schema::users::dsl::*;

    let user_name_qp = hm.0.get("name").map(|val| val.as_str());

    match user_name_qp {
        Some(u_name) => {
            let mut conn = state.db_pool.get_conn();
            let res =
                select(exists(users.filter(user_name.eq(u_name)))).get_result::<bool>(&mut conn);

            match res {
                Ok(val) => {
                    if val {
                        (StatusCode::OK, json_msg("User Exists."))
                    } else {
                        (StatusCode::NOT_FOUND, json_msg("User Does Not Exist."))
                    }
                }
                Err(e) => {
                    tracing::error!(?e, "User Exists query failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json_msg("Internal Error"),
                    )
                }
            }
        }
        None => (
            StatusCode::BAD_REQUEST,
            json_msg("name query parameter required."),
        ),
    }
}

pub async fn admin_list_users(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id_admin): UserIdExtractor,
) -> AppResult<Json<Vec<User>>> {
    use crate::schema::users::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let filt = users.filter(id.eq(user_id_admin).and(is_admin.eq(true)));

    let user_is_admin: bool = select(exists(filt)).get_result(&mut conn)?;

    if !user_is_admin {
        return Err(custom(StatusCode::UNAUTHORIZED, "Unauthorized"));
    }

    let res = users.select(User::as_select()).load::<User>(&mut conn)?;

    Ok(Json(res))
}
