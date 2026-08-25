//! Worker monitor: retry policy, dead-letter namespace, graceful shutdown.

use std::future::Future;
use std::io;

use apalis::layers::retry::{RetryLayer, RetryPolicy};
use apalis::layers::WorkerBuilderExt;
use apalis::prelude::{Monitor, WorkerBuilder, WorkerFactoryFn};
use apalis_redis::{Config, RedisStorage};

use crate::health_ping::{process_health_ping, HealthPing};
use crate::redis::RedisConn;
use crate::thumbnail::{process_thumbnail_stub, ThumbnailJob};
use crate::JobsError;

/// Redis key namespace for the sample health-ping queue.
pub const HEALTH_PING_NAMESPACE: &str = "voxnexus::health_ping";

/// Instant retries after the first failure (tower `RetryPolicy`).
pub const HEALTH_PING_RETRIES: usize = 3;

/// Redis key namespace for attachment thumbnail jobs.
pub const THUMBNAIL_NAMESPACE: &str = "voxnexus::thumbnail";

/// Instant retries after the first failure for thumbnails.
pub const THUMBNAIL_RETRIES: usize = 3;

/// Build [`RedisStorage`] for [`HealthPing`] with the shared namespace.
#[must_use]
pub fn health_ping_storage(conn: RedisConn) -> RedisStorage<HealthPing> {
    let config = Config::default().set_namespace(HEALTH_PING_NAMESPACE);
    RedisStorage::new_with_config(conn, config)
}

/// Build [`RedisStorage`] for [`ThumbnailJob`].
#[must_use]
pub fn thumbnail_storage(conn: RedisConn) -> RedisStorage<ThumbnailJob> {
    let config = Config::default().set_namespace(THUMBNAIL_NAMESPACE);
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

/// Run the stub thumbnail worker (logs only). Prefer the server Data-backed worker.
///
/// # Errors
///
/// Returns when the monitor fails to shut down cleanly.
pub async fn run_thumbnail_stub_workers<S>(
    storage: RedisStorage<ThumbnailJob>,
    shutdown: S,
) -> Result<(), JobsError>
where
    S: Future<Output = ()> + Send + 'static,
{
    let worker = WorkerBuilder::new("voxnexus-thumbnail")
        .layer(RetryLayer::new(RetryPolicy::retries(THUMBNAIL_RETRIES)))
        .enable_tracing()
        .backend(storage)
        .build_fn(process_thumbnail_stub);

    Monitor::new()
        .register(worker)
        .run_with_signal(async move {
            shutdown.await;
            Ok::<(), io::Error>(())
        })
        .await
        .map_err(JobsError::from)
}
