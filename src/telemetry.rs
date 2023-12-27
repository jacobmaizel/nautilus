pub mod metrics;
pub mod traces;
use crate::settings;
use anyhow::{Context, Result};
use gcp_auth::{AuthenticationManager, CustomServiceAccount};
use opentelemetry::{global, trace::TracerProvider as _, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    runtime::Tokio,
    trace::{self, TracerProvider},
    Resource,
};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use opentelemetry_stackdriver::{GcpAuthorizer, StackDriverExporter};
use tracing_stackdriver::CloudTraceConfiguration;
use tracing_subscriber::{layer::SubscriberExt, prelude::*, EnvFilter, Registry};
use ureq::json;

// https://gist.github.com/djc/ba2162c537098ac983c5294a6c1753f5
// https://docs.rs/opentelemetry-stackdriver/latest/opentelemetry_stackdriver/index.html#

pub async fn init_telemetry(tracing_config: settings::TracingConfig) -> Result<()> {
    let current_env = std::env::var("NAUTILUS_ENVIRONMENT").unwrap();

    match current_env.as_str() {
        "production" => prod_telemetry(tracing_config).await?,
        "development" => dev_telemetry(tracing_config)?,
        _ => {
            print!("No telemetry initialized for this environment");
        }
    }

    Ok(())
}

async fn prod_telemetry(tracing_config: settings::TracingConfig) -> anyhow::Result<()> {
    const CLOUD_TRACE_RATE: f64 = 0.5;

    let subscriber = Registry::default();
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(tracing_config.rust_log));

    let resource = Resource::new(vec![KeyValue::new(
        SERVICE_NAME,
        tracing_config.resource_name.clone(),
    )]);

    let creds = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
        .context("couldnt get app creds from env")?;

    let json_creds = json!(creds);

    let sa = CustomServiceAccount::from_json(json_creds.as_str().unwrap())
        .context("failed to convert from json")?;

    let auth_man = AuthenticationManager::try_from(sa).unwrap();

    let authorizer =
        GcpAuthorizer::from_gcp_auth(auth_man, tracing_config.gcloud_project_id.clone());

    let (exporter, driver) = StackDriverExporter::builder()
        // .log_context(log_context)
        .build(authorizer)
        .await
        .unwrap();

    tokio::spawn(driver);

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter.clone(), Tokio)
        .with_config(
            trace::Config {
                sampler: Box::new(trace::Sampler::TraceIdRatioBased(CLOUD_TRACE_RATE)),
                ..Default::default()
            }
            .with_resource(resource),
        )
        .build();

    let p_layer =
        tracing_opentelemetry::layer().with_tracer(provider.tracer(tracing_config.resource_name));

    global::set_tracer_provider(provider);
    global::set_text_map_propagator(TraceContextPropagator::new());
    // global::set_error_handler(|error| error!(error = format!("{error:#}"), "otel error pls
    // help")) .context("set error handler")?;

    let cloud_trace = tracing_stackdriver::layer().with_cloud_trace(CloudTraceConfiguration {
        project_id: tracing_config.gcloud_project_id,
    });

    let fmt_layer = tracing_subscriber::fmt::layer().json();

    let _ = subscriber
        .with(fmt_layer)
        .with(p_layer)
        .with(env_filter)
        .with(cloud_trace)
        .try_init()
        .context("Failed to init tracing subscriber");

    Ok(())
}

fn dev_telemetry(tracing_config: settings::TracingConfig) -> anyhow::Result<()> {
    let subscriber = Registry::default();

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(tracing_config.rust_log));

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(tracing_config.exporter_url);

    let resource = Resource::new(vec![KeyValue::new(
        SERVICE_NAME,
        tracing_config.resource_name,
    )]);

    let otlp_pipeline = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(opentelemetry_sdk::trace::config().with_resource(resource))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("FAILED TO CREATE TRACER");

    global::set_tracer_provider(otlp_pipeline.provider().unwrap());

    let otlp_layer = tracing_opentelemetry::layer().with_tracer(otlp_pipeline);

    global::set_text_map_propagator(TraceContextPropagator::new());
    // global::set_error_handler(|error| error!(error = format!("{error:#}"), "otel error pls
    // help"))     .context("set error handler")?;
    //
    let time_fmt = "%b %-d, %-I:%M:%S%.3f";

    // Configure a custom event formatter
    let format = tracing_subscriber::fmt::format()
        .with_level(true) //  include levels in formatted output
        .with_target(false) // don't include targets
        .with_thread_names(false) // include the name of the current thread
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            time_fmt.into(),
        ))
        .compact(); // use the `Compact` formatting style.

    let fmt_layer = tracing_subscriber::fmt::layer().event_format(format);

    let _ = subscriber
        .with(otlp_layer)
        .with(fmt_layer)
        .with(env_filter)
        .try_init()
        .context("Failed to init tracing subscriber");

    Ok(())
}
