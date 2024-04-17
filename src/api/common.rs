use crate::error::{bad_request, BoxedAppError};
use http::StatusCode;

pub async fn healthcheck() -> StatusCode {
    StatusCode::OK
}

pub async fn api_fallback() -> BoxedAppError {
    bad_request("Not Found")
}
