//! Redis-backed Apalis job workers (Feature Task F008J).

mod error;
mod health_ping;
mod redis;
mod worker;

pub use error::JobsError;
pub use health_ping::{enqueue_health_ping, process_health_ping, HealthPing};
pub use redis::{connect, ping, test_redis_url, RedisConn, TEST_REDIS_URL_ENV};
pub use worker::{
    dead_letter_key, health_ping_storage, run_health_ping_workers, HEALTH_PING_NAMESPACE,
    HEALTH_PING_RETRIES,
};

pub const CRATE_NAME: &str = "voxnexus-jobs";
