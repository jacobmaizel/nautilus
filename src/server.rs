use crate::{api::v1_routes, auth::auth_middleware, telemetry};
use anyhow::Result;
use axum::{
    middleware::{from_fn_with_state, map_request},
    routing::get,
    Json, Router,
};
use dotenv::dotenv;
use http::{HeaderValue, Method};
use hyper::StatusCode;
use moka::sync::Cache;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::{RequestBodyTimeoutLayer, TimeoutLayer},
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing_core::Level;

#[derive(Clone)]
pub struct AppState {
    pub settings: crate::settings::Settings,
    pub db_pool: crate::db::Db,
    pub cache: moka::sync::Cache<String, String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let db_url = std::env::var("DB_URL").expect("DATABASE_URL must be set");

        let settings = crate::settings::Settings::new().expect("Failed to load settings");
        let pool = crate::db::Db::new(db_url);

        pool.run_migrations().expect("Failed to run migrations!");

        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(60 * 60))
            .time_to_idle(Duration::from_secs(5 * 60))
            .build();

        AppState {
            settings,
            db_pool: pool,
            cache,
        }
    }

    pub fn new_test() -> Self {
        todo!("set this up ")
    }
}

async fn healthcheck() -> StatusCode {
    StatusCode::OK
}

async fn api_fallback() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "message": "Not Found" })),
    )
}

#[allow(dead_code)]
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // #[cfg(not(unix))]
    // let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

fn cors_layer(state: Arc<AppState>) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::PATCH,
            Method::PUT,
        ])
        .allow_origin(
            state
                .settings
                .server
                .allowed_origins
                .clone()
                .split(',')
                .map(|origin| origin.parse::<HeaderValue>().unwrap())
                .collect::<Vec<_>>(),
        );

    cors
}
// https://github.com/rust-lang/crates.io/blob/24503de6c91db0d6d47d3aac785994a0101ba80d/src/middleware.rs#L109
pub fn conditional_layer<L, F: FnOnce() -> L>(
    condition: bool,
    layer: F,
) -> axum_extra::either::Either<L, tower::layer::util::Identity> {
    axum_extra::middleware::option_layer(condition.then(layer))
}

fn app(state: Arc<AppState>) -> Router {
    let trace_layer = ServiceBuilder::new().layer(
        TraceLayer::new_for_http()
            .make_span_with(telemetry::traces::make_span)
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Millis),
            ),
    );

    axum::Router::new()
        .nest("/v1", v1_routes())
        .route_layer(from_fn_with_state(state.clone(), auth_middleware))
        .layer(trace_layer)
        // .layer(conditional_layer(state.settings.tracing.enabled, || {
        //     trace_layer
        // }))
        .layer(map_request(telemetry::traces::record_trace_id))
        .with_state(Arc::clone(&state))
        .route("/", get(healthcheck))
        .fallback(api_fallback)
        .layer(cors_layer(state.clone()))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(RequestBodyTimeoutLayer::new(Duration::from_secs(30)))
}

#[allow(dead_code)]
fn test_app() -> Router {
    axum::Router::new()
        .route("/", get(healthcheck))
        .fallback(api_fallback)
}

pub async fn start() -> Result<()> {
    dotenv().ok();

    let state = Arc::new(AppState::new());

    let port = std::env::var("PORT").expect("PORT env var must be set");

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        state.settings.server.address.clone(),
        port
    ))
    .await?;

    // start a test listener on port 8080

    telemetry::init_telemetry(state.settings.tracing.clone()).await?;
    tracing::info!("Initializing telemetry");

    tracing::info!("Nautilus Ready on port {port}");

    let app = app(state);

    // let listener = tokio::net::TcpListener::bind(format!("{}:{}", "0.0.0.0", "5050"))
    //     .await
    //     .unwrap();
    // let app = test_app();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    // .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    Ok(())
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{self, Request},
    };
    use serde_json::{json, Value};
    use std::net::{SocketAddr, TcpListener};
    use tower::ServiceExt; // for `app.oneshot()`

    #[tokio::test]
    async fn test_healthcheck() {
        let app = test_app();

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK)
    }
}
