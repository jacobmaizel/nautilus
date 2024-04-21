use crate::{
    db::{
        models::notification::{NewNotification, Notification},
        users::guard_admin,
    },
    error::{not_found, unauthorized},
    pagination::*,
    server::AppState,
    types::AppResult,
    util::extractors::{JsonExtractor, Path, QueryExtractor, UserIdExtractor},
};
use axum::{extract::State, routing::*, Json};
use diesel::{dsl::exists, insert_into, prelude::*, select, update};
use std::sync::Arc;

pub fn notification_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/",
            get(list_notifications)
                .post(create_notification)
                .delete(delete_notifications),
        )
        .route(
            "/:notification_id",
            put(update_notification).delete(delete_notification),
        )
}

pub async fn list_notifications(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id): UserIdExtractor,
    pagination: QueryExtractor<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<Notification>>> {
    use crate::schema::notifications::dsl as ndsl;

    let mut conn = state.db_pool.get_conn();

    let base = ndsl::notifications.into_boxed();

    let query = base
        .order_by(ndsl::created_at.desc())
        .filter(ndsl::user_id.eq(user_id))
        .select(Notification::as_select())
        .pages_pagination(PaginationOptions::new(pagination.0)?);

    let data: Paginated<Notification> = query.load(&mut conn)?;

    Ok(Json(data.into()))
}

async fn create_notification(
    State(state): State<Arc<AppState>>,
    JsonExtractor(body): JsonExtractor<NewNotification>,
) -> AppResult<Json<Notification>> {
    use crate::schema::notifications::dsl as ndsl;

    let mut conn = state.db_pool.get_conn();

    let res = insert_into(ndsl::notifications)
        .values(&body)
        .returning(Notification::as_returning())
        .get_result::<Notification>(&mut conn)?;

    Ok(Json(res))
}

async fn update_notification(
    State(state): State<Arc<AppState>>,
    Path(notification_id): Path<uuid::Uuid>,
    JsonExtractor(body): JsonExtractor<NewNotification>,
) -> AppResult<Json<Notification>> {
    use crate::schema::notifications::dsl as ndsl;

    let mut conn = state.db_pool.get_conn();

    let res = update(ndsl::notifications)
        .filter(ndsl::id.eq(notification_id))
        .set(&body)
        .returning(Notification::as_returning())
        .get_result::<Notification>(&mut conn)?;

    Ok(Json(res))
}

async fn delete_notifications(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(req_user_id): UserIdExtractor,
) -> AppResult<Json<serde_json::Value>> {
    use crate::schema::notifications::dsl::*;

    let mut conn = state.db_pool.get_conn();

    guard_admin(req_user_id, &mut conn)?;

    let rows: usize = diesel::delete(notifications).execute(&mut conn)?;

    Ok(Json(serde_json::json!({"deleted": rows.to_string()})))
}

async fn delete_notification(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(req_user_id): UserIdExtractor,
    Path(notification_id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    use crate::schema::notifications::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let filt = notifications.filter(user_id.eq(req_user_id).and(id.eq(notification_id)));

    let noti_exists: bool = select(exists(filt)).get_result(&mut conn)?;

    if !noti_exists {
        return Err(not_found());
    }

    let rows: usize = diesel::delete(filt).execute(&mut conn)?;

    Ok(Json(serde_json::json!({"deleted": rows.to_string()})))
}
