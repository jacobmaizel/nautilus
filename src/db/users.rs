use super::{
    models::{self, user::NewUser},
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

pub fn create_test_user(conn: &mut DbConnection) -> anyhow::Result<uuid::Uuid> {
    let nu: NewUser = NewUser {
        user_type: models::user::UserType::User,
        is_admin: false,
        onboarding_completed: false,
        first_name: "Test".to_string(),
        last_name: "Testlast".to_string(),
        user_name: "testuser".to_string(),
        email: "testuser@gmail.com".to_string(),
        phone_number: "123456789".to_string(),
        provider_id: "testprovider".to_string(),
        image: "".to_string(),
        birthday: None,
        goals: "".to_string(),
        weight: 0,
        training_approach: "".to_string(),
        training_years: 0,
        training_specializations: "".to_string(),
        bio: "".to_string(),
        gender: "".to_string(),
        beta_access: false,
    };

    use crate::schema::users::dsl::*;

    let created_user = nu
        .insert_into(users)
        .returning(id)
        .get_result::<uuid::Uuid>(conn)
        .unwrap();

    Ok(created_user)
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
