use crate::config::{CONFIG, LogFormat};
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
            KeyValue::new(SERVICE_INSTANCE_ID, cfg.instance_id.as_ref()),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(
                DEPLOYMENT_ENVIRONMENT_NAME,
                cfg.deployment_environment_name.as_ref(),
            ),
        ])
        .build()
}
/// Initialize OpenTelemetry trace
fn init_traces() -> Result<SdkTracerProvider, Box<dyn std::error::Error>> {
    let cfg = &CONFIG.otlp;
    global::set_text_map_propagator(TraceContextPropagator::new());
    let endpoint = cfg.exporter_endpoint.as_ref().ok_or_else(|| {
        Box::<dyn std::error::Error>::from(crate::error::BotError::Config(
            "OTLP exporter endpoint not configured. Disable OTLP metrics.".to_string(),
        ))
    })?;
    // Create a trace exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.as_ref())
        .with_protocol(Protocol::Grpc)
        .with_timeout(Duration::from_secs(cfg.exporter_timeout))
        .build()?;

    Ok(SdkTracerProvider::builder()
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            cfg.ratio,
        ))))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build())
}
/// Initialize OpenTelemetry metrics
fn init_metrics() -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
    let cfg = &CONFIG.otlp;
    // Check that exporter_endpoint value is specified in the config
    let endpoint = cfg.exporter_endpoint.as_ref().ok_or_else(|| {
        Box::<dyn std::error::Error>::from(crate::error::BotError::Config(
            "OTLP exporter endpoint not configured. Disable OTLP metrics.".to_string(),
        ))
    })?;
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.as_ref())
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

    // Initialize tracing and metrics providers
    let tracer_provider = init_traces().ok();
    let meter_provider = init_metrics().ok();

    // Create basic configuration for the subscriber
    let base_subscriber = tracing_subscriber::registry().with(filter_layer()?);

    if cfg.log_format == LogFormat::Json {
        let base_subscriber = base_subscriber.with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(std::io::stderr)
                .with_filter(fmt_filter()?),
        );
        // Add appropriate layers depending on initialization results
        if let (Some(meter), Some(tracer)) = (&meter_provider, &tracer_provider) {
            let tracer_instance = tracer.tracer(crate::api::types::SERVICE_NAME);
            base_subscriber
                .with(MetricsLayer::new(meter.clone()))
                .with(OpenTelemetryLayer::new(tracer_instance))
                .init();
        } else if let Some(meter) = &meter_provider {
            base_subscriber
                .with(MetricsLayer::new(meter.clone()))
                .init();
        } else if let Some(tracer) = &tracer_provider {
            let tracer_instance = tracer.tracer(crate::api::types::SERVICE_NAME);
            base_subscriber
                .with(OpenTelemetryLayer::new(tracer_instance))
                .init();
        } else {
            base_subscriber.init();
        }
    } else if cfg.log_format == LogFormat::Full {
        let base_subscriber = base_subscriber.with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_ansi(cfg.fmt_ansi)
                .with_writer(std::io::stderr)
                .with_thread_names(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_filter(fmt_filter()?),
        );
        if let (Some(meter), Some(tracer)) = (&meter_provider, &tracer_provider) {
            let tracer_instance = tracer.tracer(crate::api::types::SERVICE_NAME);
            base_subscriber
                .with(MetricsLayer::new(meter.clone()))
                .with(OpenTelemetryLayer::new(tracer_instance))
                .init();
        } else if let Some(meter) = &meter_provider {
            base_subscriber
                .with(MetricsLayer::new(meter.clone()))
                .init();
        } else if let Some(tracer) = &tracer_provider {
            let tracer_instance = tracer.tracer(crate::api::types::SERVICE_NAME);
            base_subscriber
                .with(OpenTelemetryLayer::new(tracer_instance))
                .init();
        } else {
            base_subscriber.init();
        }
    } else {
        let base_subscriber = base_subscriber.with(
            tracing_subscriber::fmt::layer()
                .with_ansi(cfg.fmt_ansi)
                .with_writer(std::io::stderr)
                .with_thread_names(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_filter(fmt_filter()?),
        );
        if let (Some(meter), Some(tracer)) = (&meter_provider, &tracer_provider) {
            let tracer_instance = tracer.tracer(crate::api::types::SERVICE_NAME);
            base_subscriber
                .with(MetricsLayer::new(meter.clone()))
                .with(OpenTelemetryLayer::new(tracer_instance))
                .init();
        } else if let Some(meter) = &meter_provider {
            base_subscriber
                .with(MetricsLayer::new(meter.clone()))
                .init();
        } else if let Some(tracer) = &tracer_provider {
            let tracer_instance = tracer.tracer(crate::api::types::SERVICE_NAME);
            base_subscriber
                .with(OpenTelemetryLayer::new(tracer_instance))
                .init();
        } else {
            base_subscriber.init();
        }
    }

    Ok(OtelGuard {
        tracer_provider,
        meter_provider,
    })
}
/// Guard for OpenTelemetry
pub struct OtelGuard {
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
}
/// Drop implementation for the OtelGuard
impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Some(provider) = &self.tracer_provider {
            if let Err(err) = provider.shutdown() {
                eprintln!("Error shutting down tracer: {err:?}");
            }
        }
        if let Some(provider) = &self.meter_provider {
            if let Err(err) = provider.shutdown() {
                eprintln!("Error shutting down meter: {err:?}");
            }
        }
    }
}
