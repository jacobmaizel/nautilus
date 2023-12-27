use crate::schema::users::dsl as user_dsl;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// check if user is admin with cool diesel stuff
// NOTE: Commenting this out for now since i dont feel like implementing the CREATE FUNCTION
// in sql right now.
// sql_function!(fn user_admin(x: Bool) -> Bool);

// type WithAdmin = Eq<user_admin::HelperType<user_dsl::is_admin>, user_admin::HelperType<bool>>;

// pub fn with_admin(admin: bool) -> WithAdmin {
//     user_admin(user_dsl::is_admin).eq(user_admin(admin))
// }
//{
// pub id: uuid::Uuid,
// pub created_at: chrono::DateTime<chrono::Utc>,
// pub first_name: String,
// pub last_name: String,
// pub user_name: String,
// pub user_type: UserType,
// // pub email: String,
// // pub phone_number: String,
// pub image: String,
// pub birthday: Option<chrono::NaiveDate>,
// pub bio: String,

// pub training_approach: String,
// pub training_years: i32,
// pub training_specializations: String,

// pub goals: String,
// }

type PublicUserColumns = (
    user_dsl::id,
    user_dsl::created_at,
    user_dsl::first_name,
    user_dsl::last_name,
    user_dsl::user_name,
    user_dsl::user_type,
    user_dsl::email,
    user_dsl::image,
    user_dsl::birthday,
    user_dsl::bio,
    user_dsl::training_approach,
    user_dsl::training_years,
    user_dsl::training_specializations,
    user_dsl::goals,
);

pub const PUBLIC_USER_COLUMNS: PublicUserColumns = (
    user_dsl::id,
    user_dsl::created_at,
    user_dsl::first_name,
    user_dsl::last_name,
    user_dsl::user_name,
    user_dsl::user_type,
    user_dsl::email,
    user_dsl::image,
    user_dsl::birthday,
    user_dsl::bio,
    user_dsl::training_approach,
    user_dsl::training_years,
    user_dsl::training_specializations,
    user_dsl::goals,
);

#[derive(diesel_derive_enum::DbEnum, Debug, Serialize, Clone, Deserialize, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::UserType"]
pub enum UserType {
    User,
    Trainer,
    Client,
}
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Identifiable, PartialEq)]
#[diesel(table_name = crate::schema::users, check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub onboarding_completed: bool,
    pub user_type: UserType,
    pub is_admin: bool,
    pub beta_access: bool,
    pub first_name: String,
    pub last_name: String,
    pub user_name: String,
    pub email: String,
    pub phone_number: String,
    pub image: String,
    pub birthday: Option<chrono::NaiveDate>,
    pub bio: String,
    pub gender: String,
    pub provider_id: String,
    pub training_approach: String,
    pub training_years: i32,
    pub training_specializations: String,
    pub goals: String,
    pub weight: i32,
}

#[derive(Insertable, Deserialize, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub user_type: UserType,
    pub is_admin: bool,
    pub onboarding_completed: bool,
    pub first_name: String,
    pub last_name: String,
    pub user_name: String,
    pub email: String,
    pub phone_number: String,
    pub image: String,
    pub birthday: Option<chrono::NaiveDate>,
    pub provider_id: String,
    pub bio: String,
    pub gender: String,
    pub beta_access: bool,

    // Trainer
    pub training_approach: String,
    pub training_years: i32,
    pub training_specializations: String,

    // Client
    pub goals: String,
    pub weight: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
pub struct PublicUser {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub first_name: String,
    pub last_name: String,
    pub user_name: String,
    pub user_type: UserType,
    pub email: String,
    // pub phone_number: String,
    pub image: String,
    pub birthday: Option<chrono::NaiveDate>,
    pub bio: String,

    pub training_approach: String,
    pub training_years: i32,
    pub training_specializations: String,

    pub goals: String,
}
