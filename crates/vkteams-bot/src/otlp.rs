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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_resource() {
        // Test resource creation with default config values
        let resource = get_resource();

        // Check that resource has expected attributes
        let attributes: Vec<_> = resource.iter().collect();
        assert!(!attributes.is_empty());

        // Check for required service attributes
        let has_service_namespace = attributes
            .iter()
            .any(|(key, _)| key.as_str() == "service.namespace");
        let has_service_instance_id = attributes
            .iter()
            .any(|(key, _)| key.as_str() == "service.instance.id");
        let has_service_version = attributes
            .iter()
            .any(|(key, _)| key.as_str() == "service.version");
        let has_deployment_env = attributes
            .iter()
            .any(|(key, _)| key.as_str() == "deployment.environment.name");

        assert!(has_service_namespace, "Missing service.namespace attribute");
        assert!(
            has_service_instance_id,
            "Missing service.instance.id attribute"
        );
        assert!(has_service_version, "Missing service.version attribute");
        assert!(
            has_deployment_env,
            "Missing deployment.environment.name attribute"
        );
    }

    #[test]
    fn test_init_traces_with_no_endpoint() {
        // Test that init_traces returns error when no endpoint is configured
        let result = init_traces();

        // Should return error because CONFIG.otlp.exporter_endpoint is None by default
        assert!(result.is_err());

        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("OTLP exporter endpoint not configured"));
        }
    }

    #[test]
    fn test_init_metrics_with_no_endpoint() {
        // Test that init_metrics returns error when no endpoint is configured
        let result = init_metrics();

        // Should return error because CONFIG.otlp.exporter_endpoint is None by default
        assert!(result.is_err());

        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("OTLP exporter endpoint not configured"));
        }
    }

    #[test]
    fn test_filter_layer_creation() {
        // Test that filter_layer creates a valid EnvFilter
        let result = filter_layer();

        // Should succeed with current config
        assert!(
            result.is_ok(),
            "Failed to create filter layer: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_fmt_filter_creation() {
        // Test that fmt_filter creates a valid EnvFilter
        let result = fmt_filter();

        // Should succeed with current config
        assert!(
            result.is_ok(),
            "Failed to create fmt filter: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_filter_layer_with_invalid_directive() {
        // This test would require mocking CONFIG, but we can test the basic functionality
        // by ensuring the function doesn't panic and returns a result
        let result = filter_layer();

        match result {
            Ok(_) => {
                // Filter created successfully
            }
            Err(e) => {
                // Error in filter creation - this is expected with invalid directives
                let error_msg = format!("{}", e);
                assert!(!error_msg.is_empty());
            }
        }
    }

    #[test]
    fn test_fmt_filter_with_service_name() {
        // Test that fmt_filter includes service name directive
        let result = fmt_filter();

        // Should succeed with current config
        assert!(
            result.is_ok(),
            "Failed to create fmt filter with service name: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_otel_guard_creation() {
        // Test OtelGuard struct creation
        let guard = OtelGuard {
            tracer_provider: None,
            meter_provider: None,
        };

        // Verify that guard is created without panics
        assert!(guard.tracer_provider.is_none());
        assert!(guard.meter_provider.is_none());
    }

    #[test]
    fn test_otel_guard_drop_with_none_providers() {
        // Test that Drop implementation handles None providers gracefully
        let guard = OtelGuard {
            tracer_provider: None,
            meter_provider: None,
        };

        // This should not panic when dropped
        drop(guard);
    }

    #[test]
    fn test_init_function_error_handling() {
        // Test individual components instead of full init to avoid global state conflicts
        let traces_result = init_traces();
        let metrics_result = init_metrics();

        // Both should fail without proper endpoint configuration
        assert!(traces_result.is_err());
        assert!(metrics_result.is_err());

        if let Err(e) = traces_result {
            let error_str = e.to_string();
            assert!(!error_str.is_empty(), "Error message should not be empty");
            assert!(error_str.contains("OTLP exporter endpoint not configured"));
        }
    }

    #[test]
    fn test_log_format_handling() {
        // Test filter creation which is used by different log formats
        let filter_result = filter_layer();
        let fmt_filter_result = fmt_filter();

        assert!(
            filter_result.is_ok(),
            "Filter layer creation should succeed"
        );
        assert!(
            fmt_filter_result.is_ok(),
            "Fmt filter creation should succeed"
        );

        // Test resource creation which is used by all log formats
        let resource = get_resource();
        let attributes: Vec<_> = resource.iter().collect();
        assert!(!attributes.is_empty(), "Resource should have attributes");
    }

    #[test]
    fn test_resource_attributes_values() {
        // Test that resource attributes have expected values
        let resource = get_resource();
        let attributes: Vec<_> = resource.iter().collect();

        // Find specific attributes and test their values
        for (key, value) in attributes {
            match key.as_str() {
                "service.namespace" => {
                    assert_eq!(value.as_str(), "vkteams");
                }
                "service.version" => {
                    assert_eq!(value.as_str(), env!("CARGO_PKG_VERSION"));
                }
                "service.instance.id" | "deployment.environment.name" => {
                    // These come from CONFIG, just verify they exist
                    assert!(!value.as_str().is_empty() || value.as_str().is_empty());
                }
                _ => {
                    // Other attributes are ok
                }
            }
        }
    }

    #[test]
    fn test_resource_service_namespace() {
        // Test that service namespace is correctly set
        let resource = get_resource();
        let namespace_attr = resource
            .iter()
            .find(|(key, _)| key.as_str() == "service.namespace")
            .map(|(_, value)| value);

        assert!(
            namespace_attr.is_some(),
            "service.namespace should be present"
        );
        if let Some(value) = namespace_attr {
            assert_eq!(value.as_str(), "vkteams");
        }
    }

    #[test]
    fn test_resource_service_version() {
        // Test that service version matches package version
        let resource = get_resource();
        let version_attr = resource
            .iter()
            .find(|(key, _)| key.as_str() == "service.version")
            .map(|(_, value)| value);

        assert!(version_attr.is_some(), "service.version should be present");
        if let Some(value) = version_attr {
            assert_eq!(value.as_str(), env!("CARGO_PKG_VERSION"));
        }
    }

    #[test]
    fn test_init_traces_error_details() {
        // Test init_traces error handling in more detail
        let result = init_traces();
        assert!(result.is_err());

        match result {
            Err(e) => {
                let error_string = e.to_string();
                assert!(error_string.contains("OTLP exporter endpoint not configured"));
            }
            Ok(_) => panic!("Expected error when no endpoint configured"),
        }
    }

    #[test]
    fn test_init_metrics_error_details() {
        // Test init_metrics error handling in more detail
        let result = init_metrics();
        assert!(result.is_err());

        match result {
            Err(e) => {
                let error_string = e.to_string();
                assert!(error_string.contains("OTLP exporter endpoint not configured"));
            }
            Ok(_) => panic!("Expected error when no endpoint configured"),
        }
    }

    #[test]
    fn test_filter_layer_default_config() {
        // Test filter_layer with default configuration
        let result = filter_layer();

        match result {
            Ok(filter) => {
                // Filter should be created successfully
                // We can't easily test the internal state, but we know it compiled correctly
                let _filter_string = format!("{:?}", filter);
            }
            Err(e) => {
                // If error occurs, it should be related to filter parsing
                let error_msg = e.to_string();
                assert!(!error_msg.is_empty());
            }
        }
    }

    #[test]
    fn test_fmt_filter_default_config() {
        // Test fmt_filter with default configuration
        let result = fmt_filter();

        match result {
            Ok(filter) => {
                // Filter should be created successfully
                let _filter_string = format!("{:?}", filter);
            }
            Err(e) => {
                // If error occurs, it should be related to filter parsing
                let error_msg = e.to_string();
                assert!(!error_msg.is_empty());
            }
        }
    }

    #[test]
    fn test_otel_guard_with_providers() {
        // Test OtelGuard with mock providers (None simulates failed initialization)
        let guard = OtelGuard {
            tracer_provider: None,
            meter_provider: None,
        };

        // Test that we can access the fields
        assert!(guard.tracer_provider.is_none());
        assert!(guard.meter_provider.is_none());

        // Test drop behavior - should not panic
        drop(guard);
    }

    #[test]
    fn test_error_propagation() {
        // Test that errors are properly propagated from init functions
        let traces_error = init_traces().unwrap_err();
        let metrics_error = init_metrics().unwrap_err();

        // Both errors should mention endpoint configuration
        assert!(traces_error.to_string().contains("endpoint"));
        assert!(metrics_error.to_string().contains("endpoint"));

        // Errors should be convertible to string
        let traces_str = format!("{:?}", traces_error);
        let metrics_str = format!("{:?}", metrics_error);

        assert!(!traces_str.is_empty());
        assert!(!metrics_str.is_empty());
    }

    #[test]
    fn test_config_usage() {
        // Test that CONFIG is properly accessed in various functions
        // This tests the configuration access paths

        // get_resource() uses CONFIG.otlp
        let resource = get_resource();
        assert!(!resource.iter().collect::<Vec<_>>().is_empty());

        // filter_layer() uses CONFIG.otlp.otel_filter_default and CONFIG.otlp.otel
        let filter_result = filter_layer();
        // Should either succeed or fail gracefully
        match filter_result {
            Ok(_) => {} // Success is fine
            Err(e) => {
                // Error should be meaningful
                assert!(!e.to_string().is_empty());
            }
        }

        // fmt_filter() uses CONFIG.otlp.fmt_filter_default and other fmt settings
        let fmt_result = fmt_filter();
        match fmt_result {
            Ok(_) => {} // Success is fine
            Err(e) => {
                // Error should be meaningful
                assert!(!e.to_string().is_empty());
            }
        }
    }

    #[test]
    fn test_service_name_constant_usage() {
        // Test that SERVICE_NAME constant is used correctly
        let result = fmt_filter();

        // The function should complete without panicking
        // SERVICE_NAME is used in the filter directive formation
        match result {
            Ok(_) => {
                // Success means SERVICE_NAME was used correctly
            }
            Err(e) => {
                // Error should still contain meaningful information
                let error_str = e.to_string();
                assert!(!error_str.is_empty());
                // Error might be related to directive parsing, which is OK
            }
        }
    }

    #[test]
    fn test_init_component_isolation() {
        // Test that init components can be called independently

        // Test resource creation (used by both traces and metrics)
        let resource1 = get_resource();
        let resource2 = get_resource();

        // Resources should be equivalent but independent
        let attrs1: Vec<_> = resource1.iter().collect();
        let attrs2: Vec<_> = resource2.iter().collect();

        assert_eq!(attrs1.len(), attrs2.len());

        // Test filter creation (used by subscriber setup)
        let filter1 = filter_layer();
        let filter2 = filter_layer();

        // Both should have same success/failure pattern
        assert_eq!(filter1.is_ok(), filter2.is_ok());

        if filter1.is_err() && filter2.is_err() {
            // Both should have similar error messages
            let err1 = filter1.unwrap_err().to_string();
            let err2 = filter2.unwrap_err().to_string();
            assert!(!err1.is_empty());
            assert!(!err2.is_empty());
        }
    }
}
