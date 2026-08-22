//! Job / Redis errors.

use std::io;

/// Failures connecting to Redis or running workers.
#[derive(Debug, thiserror::Error)]
pub enum JobsError {
    #[error("Redis error: {0}")]
    Redis(#[from] apalis_redis::RedisError),
    #[error("job worker error: {0}")]
    Worker(#[from] io::Error),
    #[error("job enqueue failed: {0}")]
    Enqueue(String),
    #[error("{0}")]
    Message(String),
}
