use crate::error::BoxedAppError;
use axum::Json;
use http::StatusCode;

pub type DBResult<T> = Result<Json<T>, (StatusCode, JsonObject)>;
pub type JsonObject = Json<serde_json::Value>;

pub type AppResult<T> = Result<T, BoxedAppError>;
