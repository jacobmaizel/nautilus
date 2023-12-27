use crate::error::{custom, BoxedAppError};
use axum::{
    async_trait,
    extract::{
        path::ErrorKind,
        rejection::{JsonRejection, PathRejection, QueryRejection},
        FromRequest, FromRequestParts, Request,
    },
    http::{request::Parts, StatusCode},
    Json,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::error::Error;
pub struct JsonExtractor<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for JsonExtractor<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<JsonError>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let (status, body) = match rejection {
                    JsonRejection::BytesRejection(inner) => {
                        let status = inner.status();
                        let err = JsonError {
                            error: inner.body_text(),
                        };
                        (status, err)
                    }

                    JsonRejection::JsonDataError(inner) => {
                        let status = inner.status();
                        let err = JsonError {
                            error: inner.body_text(),
                        };
                        (status, err)
                    }
                    JsonRejection::JsonSyntaxError(inner) => {
                        let status = inner.status();
                        let err = JsonError {
                            error: inner.body_text(),
                        };
                        (status, err)
                    }
                    JsonRejection::MissingJsonContentType(inner) => {
                        let status = inner.status();
                        let err = JsonError {
                            error: inner.body_text(),
                        };
                        (status, err)
                    }
                    _ => {
                        let status = StatusCode::BAD_REQUEST;
                        let err = JsonError {
                            error: "Failed to parse JSON Body.".to_string(),
                        };
                        (status, err)
                    }
                };

                Err((status, axum::Json(body)))
            }
        }
    }
}

#[derive(Serialize)]
pub struct JsonError {
    error: String,
    // location: Option<String>,
}

/// Custom Trainton Path Extractor with better JSON Formatted responses.
pub struct Path<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for Path<T>
where
    // these trait bounds are copied from `impl FromRequest
    // for axum::extract::path::Path`
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<PathError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let (status, body) = match rejection {
                    PathRejection::FailedToDeserializePathParams(inner) => {
                        let mut status = StatusCode::BAD_REQUEST;

                        let kind = inner.into_kind();
                        let body = match &kind {
                            ErrorKind::WrongNumberOfParameters { .. } => PathError {
                                message: kind.to_string(),
                                location: None,
                            },

                            ErrorKind::ParseErrorAtKey { key, .. } => PathError {
                                message: kind.to_string(),
                                location: Some(key.clone()),
                            },

                            ErrorKind::ParseErrorAtIndex { index, .. } => PathError {
                                message: kind.to_string(),
                                location: Some(index.to_string()),
                            },

                            ErrorKind::ParseError { .. } => PathError {
                                message: kind.to_string(),
                                location: None,
                            },

                            ErrorKind::InvalidUtf8InPathParam { key } => PathError {
                                message: kind.to_string(),
                                location: Some(key.clone()),
                            },

                            ErrorKind::UnsupportedType { .. } => {
                                // this error is caused by the programmer using an unsupported type
                                // (such as nested maps) so respond with `500` instead
                                status = StatusCode::INTERNAL_SERVER_ERROR;
                                PathError {
                                    message: kind.to_string(),
                                    location: None,
                                }
                            }

                            ErrorKind::Message(msg) => PathError {
                                message: msg.clone(),
                                location: None,
                            },

                            _ => PathError {
                                message: format!("Unhandled deserialization error: {kind}"),
                                location: None,
                            },
                        };

                        (status, body)
                    }
                    PathRejection::MissingPathParams(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        PathError {
                            message: error.to_string(),
                            location: None,
                        },
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        PathError {
                            message: format!("Unhandled path rejection: {rejection}"),
                            location: None,
                        },
                    ),
                };

                Err((status, axum::Json(body)))
            }
        }
    }
}

#[derive(Serialize)]
pub struct PathError {
    message: String,
    location: Option<String>,
}

/// This extracts all of the query parameters into a `HashMap` of `String`s.
#[derive(FromRequestParts, Debug)]
pub struct QueryHmExt(
    #[from_request(via(QueryExtractor))] pub std::collections::HashMap<String, String>,
);

// We define our own `Path` extractor that customizes the error from `axum::extract::Path`
pub struct QueryExtractor<T>(pub T);

#[derive(Serialize)]
pub struct InternalQueryError {
    pub message: String,
}

#[async_trait]
impl<S, T> FromRequestParts<S> for QueryExtractor<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = BoxedAppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Query::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(e) => {
                let msg = match e {
                    QueryRejection::FailedToDeserializeQueryString(err_msg) => {
                        let err = match err_msg.source() {
                            Some(val) => val.to_string(),
                            None => "Error Not Found.".to_string(),
                        };

                        let msg = format!("Failed to parse required query params: {}", err);
                        custom(StatusCode::BAD_REQUEST, msg)
                    }
                    _ => {
                        let msg = "Failed to parse required query params";
                        custom(StatusCode::BAD_REQUEST, msg)
                    }
                };

                return Err(msg);
            }
        }
    }
}

pub struct UserIdExtractor(pub uuid::Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for UserIdExtractor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);
    // let user_id_ext: String = request
    // .extensions()
    // .get::<String>()
    // .unwrap_or(&"1234".to_string())
    // .to_string();
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(user_id) = parts.extensions.get::<uuid::Uuid>() {
            Ok(UserIdExtractor(*user_id))
        } else {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"message": "Failed to Authenticate."})),
            ))
        }
    }
}
