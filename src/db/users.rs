use super::{
    models::{self},
    DbConnection,
};
use crate::{error::unauthorized, types::AppResult};
use diesel::{dsl::exists, prelude::*, select};

pub fn get_user(
    user_id: uuid::Uuid,
    conn: &mut DbConnection,
) -> Result<models::user::User, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let user = users
        .filter(id.eq(user_id))
        .select(models::user::User::as_select())
        .first(conn)?;

    Ok(user)
}

pub fn guard_admin(user_id: uuid::Uuid, conn: &mut DbConnection) -> AppResult<()> {
    use crate::schema::users::dsl::*;

    let fil = users.filter(id.eq(user_id).and(is_admin.eq(true)));

    let user_is_admin = select(exists(fil)).get_result::<bool>(conn)?;

    match user_is_admin {
        true => Ok(()),
        false => Err(unauthorized()),
    }
}
