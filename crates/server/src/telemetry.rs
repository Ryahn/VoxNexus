//! Tracing subscriber setup.

use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use voxnexus_config::{LogFormat, LogLevel};

/// Install the process subscriber. Pretty in interactive dev, JSON in production.
///
/// # Panics
///
/// Panics if a subscriber is already installed.
pub fn init(log_level: LogLevel, log_format: LogFormat) {
    let filter = EnvFilter::new(log_level.to_string());

    if log_format.json_on_stderr() {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                layer()
                    .json()
                    .flatten_event(true)
                    .with_current_span(true)
                    .with_span_list(false),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(layer().pretty())
            .init();
    }
}
