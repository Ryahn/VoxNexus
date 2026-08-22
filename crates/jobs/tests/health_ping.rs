use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use apalis::layers::retry::{RetryLayer, RetryPolicy};
use apalis::prelude::{Data, Error as ApalisError, Monitor, WorkerBuilder, WorkerFactoryFn};
use apalis_redis::RedisStorage;
use voxnexus_jobs::{
    connect, enqueue_health_ping, health_ping_storage, ping, process_health_ping, test_redis_url,
    HealthPing, HEALTH_PING_RETRIES, TEST_REDIS_URL_ENV,
};

#[tokio::test]
async fn enqueue_and_worker_process_health_ping() {
    let Some(url) = test_redis_url() else {
        eprintln!("skipping: set {TEST_REDIS_URL_ENV} for live Redis tests");
        return;
    };

    let conn = connect(&url).await.expect("connect redis");
    ping(&conn).await.expect("ping");

    let done = Arc::new(AtomicBool::new(false));
    let done_flag = Arc::clone(&done);
    let mut storage = health_ping_storage(conn.clone());
    let worker_storage = storage.clone();

    let worker = tokio::spawn(async move {
        let handler = move |job: HealthPing| {
            let done_flag = Arc::clone(&done_flag);
            async move {
                process_health_ping(job).await?;
                done_flag.store(true, Ordering::SeqCst);
                Ok::<(), ApalisError>(())
            }
        };
        let worker = WorkerBuilder::new("test-health-ping")
            .layer(RetryLayer::new(RetryPolicy::retries(HEALTH_PING_RETRIES)))
            .backend(worker_storage)
            .build_fn(handler);
        let _ = Monitor::new()
            .register(worker)
            .run_with_signal(async {
                tokio::time::sleep(Duration::from_secs(8)).await;
                Ok::<(), std::io::Error>(())
            })
            .await;
    });

    let job = HealthPing::new();
    enqueue_health_ping(&mut storage, job)
        .await
        .expect("enqueue");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        if done.load(Ordering::SeqCst) {
            worker.abort();
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    worker.abort();
    panic!("health ping was not processed within timeout");
}

#[tokio::test]
async fn failed_job_retries_until_success() {
    let Some(url) = test_redis_url() else {
        eprintln!("skipping: set {TEST_REDIS_URL_ENV} for live Redis tests");
        return;
    };

    let conn = connect(&url).await.expect("connect redis");
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_for_worker = Arc::clone(&attempts);
    let mut storage: RedisStorage<HealthPing> = {
        let config = apalis_redis::Config::default().set_namespace("voxnexus::health_ping_retry");
        RedisStorage::new_with_config(conn, config)
    };
    let worker_storage = storage.clone();

    let worker = tokio::spawn(async move {
        async fn flaky(
            _job: HealthPing,
            attempts: Data<Arc<AtomicU32>>,
        ) -> Result<(), ApalisError> {
            let n = attempts.fetch_add(1, Ordering::SeqCst) + 1;
            if n < 3 {
                return Err(ApalisError::Failed(Arc::new(Box::new(
                    std::io::Error::other("transient"),
                ))));
            }
            Ok(())
        }

        let worker = WorkerBuilder::new("test-health-ping-retry")
            .data(attempts_for_worker)
            .layer(RetryLayer::new(RetryPolicy::retries(5)))
            .backend(worker_storage)
            .build_fn(flaky);
        let _ = Monitor::new()
            .register(worker)
            .run_with_signal(async {
                tokio::time::sleep(Duration::from_secs(8)).await;
                Ok::<(), std::io::Error>(())
            })
            .await;
    });

    enqueue_health_ping(&mut storage, HealthPing::new())
        .await
        .expect("enqueue");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        if attempts.load(Ordering::SeqCst) >= 3 {
            worker.abort();
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    worker.abort();
    panic!(
        "expected at least 3 attempts after retries, got {}",
        attempts.load(Ordering::SeqCst)
    );
}
