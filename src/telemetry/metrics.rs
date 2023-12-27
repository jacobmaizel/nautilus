// use crate::settings;
// use anyhow::{Context, Result};
// use axum::{body::Body, extract::Request};
// use opentelemetry::{global, trace::TraceContextExt, KeyValue};
// use opentelemetry_otlp::WithExportConfig;
// use opentelemetry_sdk::{propagation::TraceContextPropagator, Resource};
// use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
// use tracing::{error, info_span, Span};
// use tracing_opentelemetry::OpenTelemetrySpanExt;
// use tracing_subscriber::{self, prelude::*, EnvFilter, Registry};

// GREAT EXAMPLE OF metrics
// https://github.com/jaegertracing/jaeger/tree/main/docker-compose/monitor
//https://www.jaegertracing.io/docs/1.52/spm/
