#![allow(unused_imports)]
use axum::{body::Body, http::Request};
use opentelemetry::{global, trace::TraceContextExt};
use opentelemetry_http::HeaderExtractor;
use tracing::{info_span, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub async fn record_trace_id<B>(request: Request<B>) -> Request<B> {
    let span = Span::current();

    let trace_id = span.context().span().span_context().trace_id();
    span.record("trace_id", trace_id.to_string());

    request
}

pub fn make_span(request: &Request<Body>) -> Span {
    // let headers = request.headers();

    let method = request.method().to_string();
    let endpoint = request.uri().to_string();

    let span_name = format!("{} {}", method, endpoint);

    info_span!(
        "Nautilus",
        "otel.name" = span_name,
        "http.method" = method,
        "http.url" = endpoint,
        trace_id = tracing::field::Empty,
    )
}

// pub fn accept_trace(request: Request<Body>) -> Request<Body> {
//     // Current context, if no or invalid data is received.
//     let parent_context = global::get_text_map_propagator(|propagator| {
//         propagator.extract(&HeaderExtractor(request.headers()))
//     });
//     Span::current().set_parent(parent_context);

//     request
// }
