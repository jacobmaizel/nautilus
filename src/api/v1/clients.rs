use crate::{
    db::{
        models::{
            client::{Client, ClientWithUser, InviteStates, NewClient, PatchClient},
            client_form::{ClientForm, NewClientForm},
            notification::NewNotification,
            user::{PublicUser, User, PUBLIC_USER_COLUMNS},
        },
        users::guard_admin,
    },
    error::{bad_request, unauthorized},
    pagination::*,
    server::AppState,
    types::AppResult,
    util::extractors::{JsonExtractor, Path, QueryExtractor, QueryHmExt, UserIdExtractor},
};
use axum::{extract::State, routing::*, Json};
use diesel::{dsl::exists, insert_into, prelude::*, select};
use serde_json::Value;
use std::{str::FromStr, sync::Arc};
use ureq::json;

pub fn client_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/",
            get(list_clients)
                .post(create_client)
                .delete(admin_delete_clients),
        )
        .route(
            "/:client_id",
            get(get_client)
                .patch(patch_client)
                .delete(delete_client_by_id),
        )
        .route(
            "/:client_id/forms",
            post(create_client_form).get(get_client_form),
        )
        .route("/summary", get(get_clients_summary))
        .route("/invite", post(invite_client))
}

pub async fn list_clients(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id): UserIdExtractor,
    pagination: QueryExtractor<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<ClientWithUser>>> {
    use crate::schema::{
        clients as base_clients_schema, clients::dsl as client_dsl, users::dsl as user_dsl,
    };
    let conn = &mut state.db_pool.get_conn();

    let page_info = pagination.0;

    let query = client_dsl::clients.into_boxed();

    let query = query
        .order(client_dsl::created_at.desc())
        .filter(client_dsl::trainer_id.eq(user_id))
        .inner_join(user_dsl::users.on(client_dsl::user_id.eq(user_dsl::id.nullable())))
        .select((base_clients_schema::all_columns, PUBLIC_USER_COLUMNS))
        .pages_pagination(PaginationOptions::new(page_info)?);

    let data: Paginated<(Client, PublicUser)> = query.load(conn)?;

    let val = data
        .iter()
        .map(|(c, u)| ClientWithUser {
            client: c.clone(),
            user: u.clone(),
        })
        .collect::<Vec<_>>();

    let res = PaginatedResponse::new(val, data.total());

    Ok(Json(res))
}

pub async fn create_client(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(user_id): UserIdExtractor,
    JsonExtractor(body): JsonExtractor<NewClient>,
) -> AppResult<Json<ClientWithUser>> {
    use crate::schema::{clients::dsl as client_dsl, users::dsl as user_dsl};

    let mut conn = state.db_pool.get_conn();

    let created_client: Client = insert_into(client_dsl::clients)
        .values((&body, client_dsl::trainer_id.eq(user_id)))
        .returning(Client::as_returning())
        .get_result(&mut conn)?;

    let user = user_dsl::users
        .filter(user_dsl::id.eq(created_client.user_id.unwrap()))
        .select(PublicUser::as_select())
        .first::<PublicUser>(&mut conn)?;

    let res: ClientWithUser = ClientWithUser {
        client: created_client,
        user,
    };

    Ok(Json(res))
}

pub async fn get_clients_summary(
    State(state): State<Arc<AppState>>,

    UserIdExtractor(user_id): UserIdExtractor,
) -> AppResult<Json<Value>> {
    use crate::schema::clients::dsl as client_dsl;

    let mut conn = state.db_pool.get_conn();
    let active_clients: i64 = client_dsl::clients
        .filter(
            client_dsl::is_active
                .eq(true)
                .and(client_dsl::trainer_id.eq(user_id)),
        )
        .count()
        .get_result(&mut conn)?;

    // Percentage of time a client gives their trainer atleast 1 testimonial
    let testimonial_rate: i64 = 0;

    // Attention needed: how many of the trainers clients have status=="Attention"
    let attention_needed: i64 = client_dsl::clients
        .filter(
            client_dsl::status
                .eq("attention")
                .and(client_dsl::trainer_id.eq(user_id)),
        )
        .count()
        .get_result(&mut conn)?;

    // Satisfaction: sum of stars given to trainer by clients / total # of clients
    let satisfaction: i64 = 0;

    Ok(Json(json!({"active_clients": active_clients, 
            "attention_needed": attention_needed,
            "testimonial_rate": testimonial_rate,
            "satisfaction": satisfaction})))
}

async fn get_client(
    State(state): State<Arc<AppState>>,
    Path(user_id_path): Path<uuid::Uuid>,
) -> AppResult<Json<ClientWithUser>> {
    use crate::schema::{
        clients as base_clients_schema, clients::dsl as cd, users::dsl as user_dsl,
    };

    let base = cd::clients.into_boxed();
    let mut conn = state.db_pool.get_conn();

    let res: (Client, PublicUser) = base
        .order(cd::created_at.desc())
        .filter(cd::user_id.eq(user_id_path).or(cd::id.eq(user_id_path)))
        .inner_join(user_dsl::users.on(cd::user_id.eq(user_dsl::id.nullable())))
        .select((base_clients_schema::all_columns, PUBLIC_USER_COLUMNS))
        .first::<(Client, PublicUser)>(&mut conn)?;

    let data = ClientWithUser {
        client: res.0,
        user: res.1,
    };

    Ok(Json(data))
}

async fn invite_client(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(req_user_id): UserIdExtractor,
    hm: QueryHmExt,
) -> AppResult<Json<Client>> {
    use crate::schema::clients::dsl as cdsl;

    if let Some(Ok(user_to_invite)) = hm.0.get("user").map(|x| uuid::Uuid::from_str(x.as_str())) {
        let mut conn = state.db_pool.get_conn();

        // 2. The user cannot invite themselves to be a client

        let inviting_self = req_user_id == user_to_invite;

        if inviting_self {
            return Err(bad_request("You cannot invite yourself to be a client."));
        }

        // 0. if a user already
        // has a pending client invite to the same person, dont send.

        let same_user_filt = cdsl::clients.filter(
            cdsl::user_id
                .eq(user_to_invite)
                .and(cdsl::invite.eq(InviteStates::Pending))
                .and(cdsl::trainer_id.eq(req_user_id)),
        );

        let trying_to_inv_same_person: bool =
            select(exists(same_user_filt)).get_result(&mut conn)?;

        if trying_to_inv_same_person {
            return Err(bad_request("You have already invited this User."));
        }

        // 1. user cannot have
        // an existing client
        // with a trainer that
        // is invite accepted && status active
        let filt = cdsl::clients.filter(
            cdsl::user_id
                .eq(user_to_invite)
                .and(cdsl::is_active.eq(true))
                .and(cdsl::trainer_id.is_not_null()),
        );

        let already_has_trainer: bool = select(exists(filt)).get_result(&mut conn)?;

        match already_has_trainer {
            true => Err(bad_request("User already has an Active Trainer")),
            false => {
                // we are good to create the new client here
                let new_client: NewClient = NewClient {
                    user_id: user_to_invite,
                    is_active: false,
                    status: "".into(),
                    invite: InviteStates::Pending,
                };

                let res = insert_into(cdsl::clients)
                    .values((&new_client, cdsl::trainer_id.eq(req_user_id)))
                    .returning(Client::as_returning())
                    .get_result::<Client>(&mut conn)?;

                use crate::schema::users::dsl as udsl;

                let sending_user: User = udsl::users
                    .filter(udsl::id.eq(req_user_id))
                    .select(User::as_select())
                    .first::<User>(&mut conn)?;

                let from_name = format!("{} {}", sending_user.first_name, sending_user.last_name);
                let d: serde_json::Value = json!({
                    "image": sending_user.image,
                    "name": from_name,
                    "client_id": res.id.to_string()
                });

                let new_noti = NewNotification::new(
                    req_user_id,
                    user_to_invite,
                    "A trainer invited you to be their client".into(),
                    "Accept their invite to begin training with them!".into(),
                    "invite".into(),
                    "unread".into(),
                    Some(d),
                );

                let _ = new_noti.send(&mut conn)?;

                Ok(Json(res))
            }
        }
    } else {
        Err(bad_request("user query param is required"))
    }
}

async fn patch_client(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(req_user_id): UserIdExtractor,
    Path(client_id): Path<uuid::Uuid>,
    JsonExtractor(body): JsonExtractor<PatchClient>,
) -> AppResult<Json<Client>> {
    use crate::schema::clients::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let res = diesel::update(clients)
        .filter(id.eq(client_id))
        .filter(user_id.eq(req_user_id).or(trainer_id.eq(req_user_id)))
        .set(body)
        .returning(Client::as_returning())
        .get_result(&mut conn)?;

    Ok(Json(res))
}

async fn admin_delete_clients(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(req_user_id): UserIdExtractor,
) -> AppResult<Json<serde_json::Value>> {
    use crate::schema::clients::dsl::*;

    let mut conn = state.db_pool.get_conn();

    guard_admin(req_user_id, &mut conn)?;

    let rows: usize = diesel::delete(clients).execute(&mut conn)?;

    Ok(Json(json!({"deleted": rows.to_string()})))
}

async fn delete_client_by_id(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(req_user_id): UserIdExtractor,
    Path(client_id): Path<uuid::Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    use crate::schema::clients::dsl::*;

    let mut conn = state.db_pool.get_conn();

    let client_res: Client = clients
        .select(Client::as_select())
        .filter(id.eq(client_id))
        .first(&mut conn)?;

    // only a trainer of a client can delete the client model
    if client_res.trainer_id != Some(req_user_id) {
        return Err(unauthorized());
    }

    let rows: usize = diesel::delete(clients.filter(id.eq(client_res.id))).execute(&mut conn)?;

    Ok(Json(json!({"deleted": rows})))
}

#[axum::debug_handler]
async fn create_client_form(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(req_user_id): UserIdExtractor,
    Path(path_client_id): Path<uuid::Uuid>,
    JsonExtractor(body): JsonExtractor<NewClientForm>,
) -> AppResult<Json<ClientForm>> {
    use crate::schema::{client_forms::dsl as client_form_dsl, clients::dsl as client_dsl};

    let mut conn = state.db_pool.get_conn();
    let path_client: Client = client_dsl::clients
        .filter(client_dsl::id.eq(path_client_id))
        .select(Client::as_select())
        .first(&mut conn)?;

    // client_id from path must match the client's id that the user belongs to, only the user should
    // create forms for their own client.

    if path_client.user_id != Some(req_user_id) {
        print!("{:#?}    {:?}", path_client, req_user_id);

        return Err(unauthorized());
    }

    // client model
    let res = insert_into(client_form_dsl::client_forms)
        .values((&body, client_form_dsl::client_id.eq(path_client_id)))
        .returning(ClientForm::as_returning())
        .get_result(&mut conn)?;

    Ok(Json(res))
}

async fn get_client_form(
    State(state): State<Arc<AppState>>,
    UserIdExtractor(req_user_id): UserIdExtractor,
    Path(path_client_id): Path<uuid::Uuid>,
) -> AppResult<Json<ClientForm>> {
    use crate::schema::{client_forms::dsl as client_form_dsl, clients::dsl as client_dsl};

    let mut conn = state.db_pool.get_conn();
    let path_client: Client = client_dsl::clients
        .filter(client_dsl::id.eq(path_client_id))
        .select(Client::as_select())
        .first(&mut conn)?;

    // client_id from path must match the client's id that the user belongs to, only the user should
    // create forms for their own client.

    if path_client.user_id != Some(req_user_id) && path_client.trainer_id != Some(req_user_id) {
        return Err(unauthorized());
    }

    // client model
    let res = client_form_dsl::client_forms
        .filter(client_form_dsl::client_id.eq(path_client.id))
        .select(ClientForm::as_select())
        .first(&mut conn)?;

    Ok(Json(res))
}
