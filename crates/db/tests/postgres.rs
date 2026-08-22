use voxnexus_db::{
    connect, connect_and_migrate, ping, revert_to, test_database_url, TEST_DATABASE_URL_ENV,
};

fn skip_without_database() -> Option<String> {
    let Some(url) = test_database_url() else {
        eprintln!("skipping: set {TEST_DATABASE_URL_ENV} to a throwaway PostgreSQL 16 database");
        return None;
    };
    Some(url)
}

#[tokio::test]
async fn pool_connects_and_pings() {
    let Some(url) = skip_without_database() else {
        return;
    };
    let pool = connect(&url).await.expect("connect");
    ping(&pool).await.expect("ping");
}

#[tokio::test]
async fn migrations_apply_and_revert() {
    let Some(url) = skip_without_database() else {
        return;
    };
    let pool = connect_and_migrate(&url).await.expect("migrate up");
    ping(&pool).await.expect("ping after migrate");
    revert_to(&pool, 0).await.expect("migrate down");
    connect_and_migrate(&url).await.expect("migrate up again");
}
