use voxnexus_search::{
    probe_message_document, test_typesense_config, SearchEngine, TypesenseClient,
    COLLECTION_MESSAGES, TEST_TYPESENSE_URL_ENV,
};

#[tokio::test]
async fn typesense_upsert_search_delete_when_endpoint_configured() {
    let Some(config) = test_typesense_config() else {
        eprintln!("skipping: set {TEST_TYPESENSE_URL_ENV} for live Typesense tests");
        return;
    };
    let client = TypesenseClient::new(config).expect("client");
    client.ping().await.unwrap_or_else(|error| {
        panic!(
            "Typesense ping failed: {error}\n\
             Start Typesense on that URL (API key must match), or unset {TEST_TYPESENSE_URL_ENV}."
        );
    });
    client
        .ensure_collections()
        .await
        .expect("ensure collections");

    let id = format!("vn-probe-{}", uuid::Uuid::now_v7());
    let body = format!("voxnexus-typesense-roundtrip-{id}");
    client
        .upsert_document(COLLECTION_MESSAGES, probe_message_document(&id, &body))
        .await
        .expect("upsert");

    // Typesense search is near-real-time; brief poll for the hit.
    let mut hits = Vec::new();
    for _ in 0..20 {
        hits = client
            .search(COLLECTION_MESSAGES, &body, "body")
            .await
            .expect("search");
        if hits.iter().any(|hit| hit == &id) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(
        hits.iter().any(|hit| hit == &id),
        "expected id {id} in search hits, got {hits:?}"
    );

    client
        .delete_document(COLLECTION_MESSAGES, &id)
        .await
        .expect("delete");
    // Delete of missing id is ok.
    client
        .delete_document(COLLECTION_MESSAGES, &id)
        .await
        .expect("delete again");
}
