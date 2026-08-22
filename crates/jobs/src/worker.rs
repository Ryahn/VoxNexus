//! Worker monitor: retry policy, dead-letter namespace, graceful shutdown.

use std::future::Future;
use std::io;

use apalis::layers::retry::{RetryLayer, RetryPolicy};
use apalis::layers::WorkerBuilderExt;
use apalis::prelude::{Monitor, WorkerBuilder, WorkerFactoryFn};
use apalis_redis::{Config, RedisStorage};

use crate::health_ping::{process_health_ping, HealthPing};
use crate::redis::RedisConn;
use crate::JobsError;

/// Redis key namespace for the sample health-ping queue.
pub const HEALTH_PING_NAMESPACE: &str = "voxnexus::health_ping";

/// Instant retries after the first failure (tower `RetryPolicy`).
pub const HEALTH_PING_RETRIES: usize = 3;

/// Build [`RedisStorage`] for [`HealthPing`] with the shared namespace.
#[must_use]
pub fn health_ping_storage(conn: RedisConn) -> RedisStorage<HealthPing> {
    let config = Config::default().set_namespace(HEALTH_PING_NAMESPACE);
    RedisStorage::new_with_config(conn, config)
}

/// Redis sorted-set key where exhausted jobs land (`…:dead`).
#[must_use]
pub fn dead_letter_key() -> String {
    Config::default()
        .set_namespace(HEALTH_PING_NAMESPACE)
        .dead_jobs_set()
}

/// Run the health-ping worker until `shutdown` completes.
///
/// Exhausted retries are moved to the Redis dead-letter set for this namespace
/// (Apalis `:dead` key). Job payloads remain small IDs.
///
/// # Errors
///
/// Returns when the monitor fails to shut down cleanly.
pub async fn run_health_ping_workers<S>(
    storage: RedisStorage<HealthPing>,
    shutdown: S,
) -> Result<(), JobsError>
where
    S: Future<Output = ()> + Send + 'static,
{
    let worker = WorkerBuilder::new("voxnexus-health-ping")
        .layer(RetryLayer::new(RetryPolicy::retries(HEALTH_PING_RETRIES)))
        .enable_tracing()
        .backend(storage)
        .build_fn(process_health_ping);

    Monitor::new()
        .register(worker)
        .run_with_signal(async move {
            shutdown.await;
            Ok::<(), io::Error>(())
        })
        .await
        .map_err(JobsError::from)
}
