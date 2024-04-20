// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "intensity_choices"))]
    pub struct IntensityChoices;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "invite_states"))]
    pub struct InviteStates;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_type"))]
    pub struct UserType;
}

diesel::table! {
    betacode (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        #[max_length = 50]
        code -> Varchar,
    }
}

diesel::table! {
    certifications (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        user_id -> Nullable<Uuid>,
        #[max_length = 50]
        name -> Varchar,
        expiration -> Nullable<Date>,
    }
}

diesel::table! {
    client_forms (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        client_id -> Nullable<Uuid>,
        #[max_length = 500]
        health_history -> Varchar,
        #[max_length = 500]
        lifestyle -> Varchar,
        #[max_length = 500]
        time_availability -> Varchar,
        #[max_length = 500]
        motivation -> Varchar,
        #[max_length = 500]
        preferences -> Varchar,
        #[max_length = 500]
        extra_details -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::InviteStates;

    clients (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        user_id -> Nullable<Uuid>,
        trainer_id -> Nullable<Uuid>,
        is_active -> Bool,
        #[max_length = 50]
        status -> Varchar,
        invite -> InviteStates,
        accepted_invite_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::IntensityChoices;

    exercises (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        workout_id -> Nullable<Uuid>,
        owner_id -> Nullable<Uuid>,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 255]
        description -> Varchar,
        #[max_length = 50]
        duration -> Varchar,
        #[max_length = 50]
        reps -> Varchar,
        sets -> Int4,
        #[max_length = 50]
        rest_period -> Varchar,
        intensity -> IntensityChoices,
        #[max_length = 255]
        equipment -> Varchar,
        #[max_length = 255]
        muscle_groups -> Varchar,
        #[max_length = 255]
        image -> Varchar,
        #[max_length = 255]
        video -> Varchar,
        #[max_length = 500]
        instructions -> Varchar,
        sequence -> Int4,
        #[max_length = 255]
        slug -> Varchar,
    }
}

diesel::table! {
    feedback (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        user_id -> Nullable<Uuid>,
        #[max_length = 100]
        title -> Varchar,
        #[max_length = 500]
        description -> Varchar,
        followup -> Bool,
    }
}

diesel::table! {
    notifications (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        sender_id -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
        #[max_length = 50]
        title -> Varchar,
        #[max_length = 255]
        content -> Varchar,
        #[max_length = 50]
        category -> Varchar,
        #[max_length = 50]
        status -> Varchar,
        opened_at -> Nullable<Timestamptz>,
        data -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::IntensityChoices;

    programs (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        owner_id -> Nullable<Uuid>,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 255]
        description -> Varchar,
        #[max_length = 50]
        duration -> Varchar,
        #[max_length = 255]
        focus_areas -> Varchar,
        #[max_length = 50]
        target_audience -> Varchar,
        #[max_length = 255]
        program_image -> Varchar,
        intensity -> IntensityChoices,
        #[max_length = 255]
        slug -> Varchar,
        template -> Bool,
        client_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserType;

    users (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        onboarding_completed -> Bool,
        user_type -> UserType,
        is_admin -> Bool,
        beta_access -> Bool,
        #[max_length = 255]
        first_name -> Varchar,
        #[max_length = 255]
        last_name -> Varchar,
        #[max_length = 255]
        user_name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        phone_number -> Varchar,
        #[max_length = 255]
        image -> Varchar,
        birthday -> Nullable<Date>,
        #[max_length = 255]
        bio -> Varchar,
        #[max_length = 30]
        gender -> Varchar,
        #[max_length = 255]
        provider_id -> Varchar,
        #[max_length = 255]
        training_approach -> Varchar,
        training_years -> Int4,
        #[max_length = 255]
        training_specializations -> Varchar,
        #[max_length = 255]
        goals -> Varchar,
        weight -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::IntensityChoices;

    workouts (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        program_id -> Nullable<Uuid>,
        owner_id -> Nullable<Uuid>,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 255]
        description -> Varchar,
        #[max_length = 50]
        duration -> Varchar,
        sequence -> Int4,
        week -> Int4,
        intensity -> IntensityChoices,
        #[max_length = 50]
        workout_type -> Varchar,
        #[max_length = 255]
        equipment_needed -> Varchar,
        #[max_length = 255]
        image -> Varchar,
        #[max_length = 255]
        video -> Varchar,
        template -> Bool,
        #[max_length = 255]
        slug -> Varchar,
    }
}

diesel::joinable!(certifications -> users (user_id));
diesel::joinable!(client_forms -> clients (client_id));
diesel::joinable!(exercises -> users (owner_id));
diesel::joinable!(exercises -> workouts (workout_id));
diesel::joinable!(feedback -> users (user_id));
diesel::joinable!(programs -> clients (client_id));
diesel::joinable!(programs -> users (owner_id));
diesel::joinable!(workouts -> programs (program_id));
diesel::joinable!(workouts -> users (owner_id));

diesel::allow_tables_to_appear_in_same_query!(
    betacode,
    certifications,
    client_forms,
    clients,
    exercises,
    feedback,
    notifications,
    programs,
    users,
    workouts,
);
