use crate::{
    db::{
        models::user::{self, NewUser},
        DbConnection,
    },
    error::{internal_server_error, unauthorized, BoxedAppError},
    server::AppState,
    settings,
};
use anyhow::anyhow;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use diesel::{insert_into, prelude::*};
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tracing::error;
// Using Env Vars as Config with templates
// https://github.com/mehcode/config-rs/issues/447#issuecomment-1666885398

const JWKS_KEY: &str = "jwks";

fn get_jwks(
    domain: &str,
    cache: &moka::sync::Cache<String, String>,
) -> Result<JwkSet, anyhow::Error> {
    // Check if the jwks is in the cache
    if let Some(jwks) = cache.get(JWKS_KEY) {
        let jwks: JwkSet = serde_json::from_str(&jwks)?;
        return Ok(jwks);
    }

    // Otherwise we fetch it and store it
    let uri = format!("https://{}/.well-known/jwks.json", domain);
    let res = ureq::get(&uri).call()?.into_string()?;

    let jwks: JwkSet = serde_json::from_str(&res)?;

    cache.insert(JWKS_KEY.to_string(), serde_json::to_string(&jwks)?);

    Ok(jwks)
}

pub async fn decode_token(
    token: &str,
    _management_audience: &str,
    auth_audience: &str,
    domain: &str,
    cache: &moka::sync::Cache<String, String>,
) -> anyhow::Result<jsonwebtoken::TokenData<HashMap<String, serde_json::Value>>> {
    let jwks = get_jwks(domain, cache)?;

    let header = decode_header(token);

    let header = match header {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Error decoding token header: {:?}", e);
            return Err(anyhow!("Error decoding token header: {:?}", e));
        }
    };

    let kid = match header.kid {
        Some(k) => k,
        None => {
            tracing::error!("No kid found in token header");
            return Err(anyhow!("Invalid Token"));
        }
    };

    if let Some(j) = jwks.find(&kid) {
        match &j.algorithm {
            AlgorithmParameters::RSA(rsa) => {
                let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap();

                let mut validation = Validation::new(
                    Algorithm::from_str(j.common.key_algorithm.unwrap().to_string().as_str())
                        .unwrap(),
                );

                validation.set_audience(&[auth_audience]);

                validation.validate_exp = true;

                let decoded_token: jsonwebtoken::TokenData<HashMap<String, serde_json::Value>> =
                    decode::<HashMap<String, serde_json::Value>>(token, &decoding_key, &validation)
                        .map_err(|e| {
                            tracing::error!("Error decoding token: {:?}", e);
                            anyhow!("Invalid Token")
                        })?;

                Ok(decoded_token)
            }
            _ => unreachable!("this should be a RSA"),
        }
    } else {
        {
            tracing::error!("No matching JWK found for the given kid");
            Err(anyhow!("Invalid Token"))
        }
    }
}

pub async fn extract_user_provider_id(
    token: &str,
    management_audience: &str,
    auth_audience: &str,
    domain: &str,
    cache: &moka::sync::Cache<String, String>,
) -> anyhow::Result<String> {
    let decoded_token =
        decode_token(token, auth_audience, management_audience, domain, cache).await?;

    let sub = decoded_token
        .claims
        .get("sub")
        .ok_or(anyhow!("Invalid Token"))?
        .to_string()
        .replace('"', "");

    Ok(sub)
}

pub async fn get_auth0_management_api_bearer_token(
    settings: settings::Settings,
) -> anyhow::Result<String> {
    let uri = format!("https://{}/oauth/token", settings.auth_domain);

    let res = ureq::post(&uri)
        .send_form(&[
            ("grant_type", "client_credentials"),
            ("client_id", &settings.auth_management_client_id),
            ("client_secret", &settings.auth_management_secret),
            ("audience", &settings.auth_management_audience),
        ])
        .map_err(|e| anyhow!("Failed to get creds for management api: {:?}", e))?
        .into_string()?;

    let body: serde_json::Value = serde_json::from_str(&res)?;

    return Ok(body.get("access_token").unwrap().to_string());
}

pub async fn get_auth0_user<'a>(
    sub: &'a str,
    settings: &settings::Settings,
) -> anyhow::Result<serde_json::Value> {
    let token = get_auth0_management_api_bearer_token(settings.clone()).await?;

    let formatted_url = format!("{}users/{}", settings.auth_management_audience, sub);

    let res = ureq::get(&formatted_url)
        .set(
            "Authorization",
            &format!("Bearer {}", token).replace('"', ""),
        )
        .set("Content-Type", "application/json; charset=utf-8")
        .set("Accept", "application/json");

    let res = res
        .send_json(serde_json::json!({}))
        .map_err(|e| {
            error!("Failed to get user from management api: {:?}", e);
            anyhow!("Failed to get user from management api: {:?}", e)
        })?
        .into_string()
        .map_err(|e| anyhow!("Failed to get user from management api: {:?}", e))?;

    let body = serde_json::from_str(&res)?;

    Ok(body)
}

#[allow(unused_variables)]
#[tracing::instrument(skip_all)]
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    token: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request,
    next: Next,
) -> Result<Response, BoxedAppError> {
    // Early return in Test
    // if cfg!(test) {
    //     let user_id = create_test_user(&mut state.db_pool.get_conn()).unwrap();
    //     request.extensions_mut().insert(user_id);

    //     return Ok(next.run(request).await);
    // }

    if let Some(token) = token {
        let token = token.token();
        // Extract provider id from token
        let sub = extract_user_provider_id(
            token,
            &state.settings.auth_audience,
            &state.settings.auth_management_audience,
            &state.settings.auth_domain,
            &state.cache,
        )
        .await
        .map_err(|e| {
            error!("Error extracting provider id from token: {}", e);
            // (StatusCode::FORBIDDEN, json_msg("Unauthorized"))
            unauthorized()
        })?;

        // Get or Create User ID from DB
        let user_id = get_or_create_user_from_provider_id(
            &sub,
            &mut state.db_pool.get_conn(),
            &state.settings,
        )
        .await
        .map_err(|e| {
            error!(
                "Error getting or creating user from provider id: {}",
                e.to_string()
            );
            internal_server_error(e)
        })?;

        // Attach user id to request
        request.extensions_mut().insert(user_id);

        // Call next middleware
        Ok(next.run(request).await)
    } else {
        Err(unauthorized())
    }
}

async fn get_or_create_user_from_provider_id<'a>(
    sub: &'a str,
    conn: &mut DbConnection,
    settings: &settings::Settings,
) -> anyhow::Result<uuid::Uuid> {
    use crate::schema::users::dsl::*;

    let q = users.select(id).filter(provider_id.eq(sub));

    let q_res = q.first::<uuid::Uuid>(conn);

    // Attempt to find the user by provider_id
    match q_res {
        Ok(user_id) => {
            // User found, return early
            Ok(user_id)
        }
        Err(diesel::NotFound) => {
            // User not found, proceed to create a new user

            let auth0_user = get_auth0_user(sub, settings).await?;

            let new_db_user: NewUser = auth0_user.into();

            let created_user_id = insert_into(users)
                .values(&new_db_user)
                .returning(id)
                .get_result::<uuid::Uuid>(conn)?;

            Ok(created_user_id)
        }
        Err(e) => {
            // Handle other diesel errors
            Err(e.into())
        }
    }
}

impl From<serde_json::Value> for NewUser {
    fn from(serde_val: serde_json::Value) -> Self {
        let onboarding_complated: bool = false;
        let first_name: String = serde_val
            .get("given_name")
            .unwrap()
            .to_string()
            .replace('"', "");
        let last_name = serde_val
            .get("family_name")
            .unwrap()
            .to_string()
            .replace('"', "");

        let user_name = first_name.clone()
            + &last_name.clone()
            + &rand::random::<u32>().to_string().replace('"', "");

        let email = serde_val.get("email").unwrap().to_string().replace('"', "");
        let phone_number = String::from("");
        let image = serde_val
            .get("picture")
            .unwrap()
            .to_string()
            .replace('"', "");
        let birthday: Option<chrono::NaiveDate> = None;
        let provider_id = serde_val
            .get("user_id")
            .unwrap()
            .to_string()
            .replace('"', "");

        let is_admin = {
            email == "maizeljacob@gmail.com"
                || email == "jacobmaizel2023@gmail.com"
                || email == "jake@trainton.com"
        };

        let training_years = 0;
        let training_specializations = String::from("");
        let training_approach = String::from("");
        let goals = String::from("");
        let weight = 0;
        let gender = String::from("");
        let bio = String::from("");

        let beta_access: bool = false;

        NewUser {
            user_type: user::UserType::User,
            is_admin,
            onboarding_completed: onboarding_complated,
            first_name,
            last_name,
            user_name,
            email,
            phone_number,
            image,
            birthday,
            provider_id,
            training_years,
            training_specializations,
            training_approach,
            goals,
            weight,
            gender,
            bio,
            beta_access,
        }
    }
}
