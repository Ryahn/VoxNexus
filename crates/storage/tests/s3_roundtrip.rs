use bytes::Bytes;
use voxnexus_storage::{
    test_s3_config, ObjectKey, ObjectStore, S3ObjectStore, S3ObjectStoreConfig, StorageError,
    TEST_S3_ENDPOINT_ENV,
};

#[tokio::test]
async fn s3_put_get_delete_when_endpoint_configured() {
    let Some(config) = test_s3_config() else {
        eprintln!("skipping: set {TEST_S3_ENDPOINT_ENV} for live S3 tests");
        return;
    };
    let endpoint = config.endpoint.clone();
    let store = S3ObjectStore::new(S3ObjectStoreConfig {
        endpoint: config.endpoint,
        access_key: config.access_key,
        secret_key: config.secret_key,
        bucket: config.bucket,
        region: "us-east-1".to_owned(),
    });
    store.ensure_bucket().await.unwrap_or_else(|error| {
        let hint = if error.to_string().contains("403") {
            "HTTP 403: credentials do not match SeaweedFS IAM. Set S3_ACCESS_KEY_TEST / \
             S3_SECRET_KEY_TEST (or S3_ACCESS_KEY / S3_SECRET_KEY) to the same values as \
             config.toml / the container s3.json. Defaults are any/any."
        } else if error.to_string().contains("dispatch failure") {
            "TCP connect failed — nothing listening on that URL."
        } else {
            "Check SeaweedFS is up and credentials match."
        };
        panic!(
            "ensure bucket against {endpoint} failed: {error}\n{hint}\n\
             Unset {TEST_S3_ENDPOINT_ENV} to skip this test."
        );
    });

    let key = ObjectKey::parse(format!("vn/test/{}", uuid_like())).expect("key");
    let payload = Bytes::from_static(b"voxnexus-s3-roundtrip");
    store
        .put(key.clone(), payload.clone(), "application/octet-stream")
        .await
        .expect("put");
    let got = store.get(&key).await.expect("get");
    assert_eq!(got, payload);
    store.delete(&key).await.expect("delete");
    assert!(matches!(
        store.get(&key).await,
        Err(StorageError::NotFound(_))
    ));
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    format!("obj-{nanos}")
}
