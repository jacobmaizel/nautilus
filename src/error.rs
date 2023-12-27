use crate::types::JsonObject;
use axum::{response::IntoResponse, Json};
use core::fmt;
use diesel::result::Error as DieselError;
use http::StatusCode;
// use thiserror::Error;
use std::error::Error;
use std::{
    any::{Any, TypeId},
    borrow::Cow,
};
use tracing::error;

//////////////////////////////// CustomApiError
// https://github.com/rust-lang/crates.io/blob/ee08e191f474b1b8fa9b31f8ef32ea6dee98cc22/src/util/errors/json.rs#L53
#[derive(Debug, Clone)]
struct CustomApiError {
    status: StatusCode,
    detail: Cow<'static, str>,
}

pub type BoxedAppError = Box<dyn AppError>;

/// DEPCRECATED - use `custom` instead
pub fn api_error<E>(err: E) -> (StatusCode, JsonObject)
where
    E: std::error::Error,
{
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "message": err.to_string()})),
    )
}

pub fn json_msg(msg: &str) -> JsonObject {
    Json(serde_json::json!({ "message": msg }))
}
pub fn json_err(msg: &str) -> JsonObject {
    Json(serde_json::json!({ "error": msg }))
}

pub fn json_error(detail: &str, status: StatusCode) -> axum::response::Response {
    let json = serde_json::json!({ "errors": [{ "detail": detail }] });
    (status, Json(json)).into_response()
}

///////////////////////////////// AppError Trait
pub trait AppError: Send + fmt::Display + fmt::Debug + 'static {
    fn response(&self) -> axum::response::Response;

    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl dyn AppError {
    pub fn is<T: Any>(&self) -> bool {
        self.get_type_id() == TypeId::of::<T>()
    }
}

impl AppError for CustomApiError {
    fn response(&self) -> axum::response::Response {
        json_error(&self.detail, self.status)
    }
}

impl fmt::Display for CustomApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.detail.fmt(f)
    }
}

pub fn custom(status: StatusCode, detail: impl Into<Cow<'static, str>>) -> BoxedAppError {
    Box::new(CustomApiError {
        status,
        detail: detail.into(),
    })
}

pub fn unauthorized() -> BoxedAppError {
    custom(
        StatusCode::UNAUTHORIZED,
        "You are not authorized to perform this action",
    )
}

pub fn internal_server_error<S: ToString>(err: S) -> BoxedAppError {
    custom(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

impl AppError for BoxedAppError {
    fn response(&self) -> axum::response::Response {
        (**self).response()
    }

    fn get_type_id(&self) -> TypeId {
        (**self).get_type_id()
    }
}

fn server_error_response(_error: String) -> axum::response::Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
}

pub fn bad_request<S: ToString>(error: S) -> BoxedAppError {
    custom(StatusCode::BAD_REQUEST, error.to_string())
}

impl<E: Error + Send + 'static> AppError for E {
    fn response(&self) -> axum::response::Response {
        tracing::error!(error = %self, "Internal Server Error");

        // sentry::capture_error(self);

        server_error_response(self.to_string())
    }
}
impl From<DieselError> for BoxedAppError {
    fn from(err: DieselError) -> BoxedAppError {
        match err {
            DieselError::NotFound => not_found(),
            _ => {
                error!(?err, "Unexpected Diesel error");
                custom(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
        }
    }
}

pub fn not_found() -> BoxedAppError {
    custom(StatusCode::NOT_FOUND, "Not Found")
}

impl IntoResponse for BoxedAppError {
    fn into_response(self) -> axum::response::Response {
        self.response()
    }
}
