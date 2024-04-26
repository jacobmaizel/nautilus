use crate::{
    api::{
        common::{api_fallback, healthcheck},
        v1_routes,
    },
    auth::auth_middleware,
    telemetry,
};
use anyhow::Result;
use axum::{
    middleware::{from_fn_with_state, map_request},
    routing::get,
    Router,
};
use dotenv::dotenv;
use http::{HeaderValue, Method};
use moka::sync::Cache;
use std::{net::SocketAddr, sync::Arc, time::Duration};
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
        dotenv().ok();
        let db_url = std::env::var("DB_URL").expect("DATABASE_URL must be set");

        let settings = crate::settings::Settings::new().expect("Failed to load settings");
        let pool = crate::db::Db::new(db_url);

        if settings.environment != "test" {
            pool.run_migrations().expect("Failed to run migrations!");
        }

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

fn build_router(state: Arc<AppState>) -> Router {
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
        .with_state(Arc::clone(&state))
        .route("/", get(healthcheck))
        .layer(map_request(telemetry::traces::record_trace_id))
        .fallback(api_fallback)
        .layer(cors_layer(state.clone()))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(RequestBodyTimeoutLayer::new(Duration::from_secs(30)))
}

pub async fn start() -> Result<()> {
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

    let app = build_router(state);

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
    use crate::util::tests::*;
    use axum::{
        body::Body,
        http::{self, Request},
    };
    use serde_json::{json, Value};
    use std::net::{SocketAddr, TcpListener};
    use tower::ServiceExt; // for `app.oneshot()`

    #[tokio::test]
    async fn test_healthcheck() {
        let ctx = TestContext::default();

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = ctx.router.oneshot(req).await.unwrap();

        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST)
    }
}
