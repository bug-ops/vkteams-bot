use crate::config::CONFIG;
use opentelemetry::{KeyValue, global, trace::TracerProvider as _};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::{PeriodicReader, SdkMeterProvider},
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use opentelemetry_semantic_conventions::{
    attribute::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_VERSION},
    resource::{SERVICE_INSTANCE_ID, SERVICE_NAMESPACE},
};
use std::result::Result;
use tokio::time::Duration;
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

fn get_resource() -> Resource {
    let cfg = &CONFIG.otlp;
    Resource::builder()
        .with_service_name(cfg.instance_id.as_ref())
        .with_attributes(vec![
            KeyValue::new(SERVICE_NAMESPACE, "vkteams"),
            KeyValue::new(SERVICE_INSTANCE_ID, CONFIG.otlp.instance_id.as_ref()),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "develop"),
        ])
        .build()
}
/// Initialize OpenTelemetry trace
fn init_traces() -> Result<SdkTracerProvider, Box<dyn std::error::Error>> {
    let cfg = &CONFIG.otlp;
    global::set_text_map_propagator(TraceContextPropagator::new());
    // Create a trace exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(cfg.exporter_endpoint.as_ref())
        .with_protocol(Protocol::Grpc)
        .with_timeout(Duration::from_secs(cfg.exporter_timeout))
        .build()?;

    Ok(SdkTracerProvider::builder()
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            1.0,
        ))))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build())
}
/// Initialize OpenTelemetry metrics
fn init_metrics() -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
    let cfg = &CONFIG.otlp;
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(cfg.exporter_endpoint.as_ref())
        .with_protocol(Protocol::Grpc)
        .with_timeout(Duration::from_secs(cfg.exporter_timeout))
        .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
        .build()?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(cfg.exporter_metric_interval))
        .build();

    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader)
        // .with_reader(stdout_reader)
        .with_resource(get_resource())
        .build();

    global::set_meter_provider(meter_provider.clone());
    Ok(meter_provider)
}
/// Create a filter for the OpenTelemetry subscriber
fn filter_layer() -> Result<EnvFilter, Box<dyn std::error::Error>> {
    let cfg = &CONFIG.otlp;
    // Create a filter for the OpenTelemetry subscriber
    let mut filter = tracing_subscriber::EnvFilter::new(cfg.otel_filter_default.as_ref());
    for otel in &cfg.otel {
        filter = filter.add_directive(otel.otel_filter_directive.parse()?);
    }

    Ok(filter)
}
/// Create a filter for the subscriber
fn fmt_filter() -> Result<EnvFilter, Box<dyn std::error::Error>> {
    let cfg = &CONFIG.otlp;
    // Create a filter for the subscriber
    let mut fmt_filter = tracing_subscriber::EnvFilter::new(cfg.fmt_filter_default.as_ref())
        .add_directive(
            format!(
                "{}={}",
                crate::api::types::SERVICE_NAME,
                cfg.fmt_filter_self_directive
            )
            .parse()?,
        );
    for fmt in &cfg.fmt {
        fmt_filter = fmt_filter.add_directive(fmt.fmt_filter_directive.parse()?);
    }

    Ok(fmt_filter)
}
/// Initialize OpenTelemetry
pub fn init() -> Result<OtelGuard, Box<dyn std::error::Error>> {
    let cfg = &CONFIG.otlp;
    let tracer_provider = init_traces()?;
    let meter_provider = init_metrics()?;
    // Create a formatting layer with the filter
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(cfg.fmt_ansi)
        .with_thread_names(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_filter(fmt_filter()?);
    // Create a tracer
    let tracer = tracer_provider.tracer(crate::api::types::SERVICE_NAME);
    // Create a subscriber with the filter and formatting layer
    tracing_subscriber::registry()
        .with(filter_layer()?)
        .with(fmt_layer)
        .with(MetricsLayer::new(meter_provider.clone()))
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    Ok(OtelGuard {
        tracer_provider,
        meter_provider,
    })
}
/// Guard for OpenTelemetry
pub struct OtelGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
}
/// Drop implementation for the OtelGuard
impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("{err:?}");
        }
        if let Err(err) = self.meter_provider.shutdown() {
            eprintln!("{err:?}");
        }
    }
}
