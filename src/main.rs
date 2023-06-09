use chrono::prelude::{DateTime, Utc};
use opentelemetry_otlp::WithExportConfig;
use tracing::{warn, warn_span};
use tracing_subscriber::{
    fmt::{format::Writer, time::FormatTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

struct CustomTime;

impl FormatTime for CustomTime {
    fn format_time(&self, w: &mut Writer<'_>) -> core::fmt::Result {
        let dt: DateTime<Utc> = std::time::SystemTime::now().into();
        write!(w, "{}", dt.format("%H:%M:%S%.3f"))
    }
}

/// Configure and init tracing for executables
pub fn init_tracing(service_name: &str) {
    let mut fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_timer(CustomTime);
    fmt_layer.set_ansi(false);

    let reg = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt_layer);

    let ot_url = "https://tempo-prod-04-prod-us-east-0.grafana.net";
    // let ot_url = "https://tempo-prod-04-prod-us-east-0.grafana.net/tempo";
    // let ot_url = "https://otlp-gateway-prod-us-east-0.grafana.net/otlp";
    let ot_auth = "Basic <base64 encoded instance_id:key>";

    let mut headers = tonic::metadata::MetadataMap::new();
    headers.insert("authorization", ot_auth.parse().unwrap());
    // let headers = std::collections::HashMap::from([("Authorization".into(), ot_auth.into())]);
    let exporter = opentelemetry_otlp::new_exporter()
        // .http()
        .tonic()
        .with_endpoint(ot_url)
        .with_metadata(headers);
    // .with_headers(headers);
    let otlp_tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
            opentelemetry::sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                "service.name",
                service_name.to_owned(),
            )]),
        ))
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
        .unwrap();
    let tracing_layer = tracing_opentelemetry::layer().with_tracer(otlp_tracer);

    reg.with(tracing_layer).init();
}

pub fn close_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    init_tracing("hello_srv");
    warn_span!("hello span!").in_scope(|| {
        warn!("Hello event!");
    });
    opentelemetry::global::shutdown_tracer_provider();
}
