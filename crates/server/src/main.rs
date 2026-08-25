use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use voxnexus_auth::{ensure_instance, InstanceSeed};
use voxnexus_config::Config;
use voxnexus_domain::{CommunityCreationMode, RegistrationMode};
use voxnexus_jobs::{
    connect, health_ping_storage, ping, run_health_ping_workers, run_thumbnail_stub_workers,
    thumbnail_storage, RedisConn,
};
use voxnexus_permissions::PermissionCache;
use voxnexus_search::{SearchEngine, TypesenseClient, TypesenseConfig};
use voxnexus_storage::{ObjectStore, S3ObjectStore, S3ObjectStoreConfig};

#[tokio::main]
async fn main() {
    if let Err(code) = run().await {
        std::process::exit(code);
    }
}

#[allow(clippy::too_many_lines)]
async fn run() -> Result<(), i32> {
    let config = Config::load().map_err(|error| {
        eprintln!("voxnexus: {error}");
        1
    })?;

    voxnexus::telemetry::init(config.log_level, config.log_format);

    tracing::info!(
        public_url = %config.public_url,
        listen_addr = %config.listen_addr,
        log_level = %config.log_level,
        log_format = %config.log_format,
        metrics_enabled = config.metrics_enabled,
        gateway_allow_unauth = config.gateway_allow_unauth,
        "configuration loaded"
    );

    let pool = voxnexus_db::connect_and_migrate(config.database_url.as_str())
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "database startup failed");
            1
        })?;
    voxnexus_db::ping(&pool).await.map_err(|error| {
        tracing::error!(error = %error, "database ping failed");
        1
    })?;
    tracing::info!("database ready");

    if let (Some(email), Some(password)) = (
        config.bootstrap_admin_email.as_ref(),
        config.bootstrap_admin_password.as_ref(),
    ) {
        match voxnexus_auth::bootstrap_instance_admin(&pool, email, password.expose()).await {
            Ok(voxnexus_auth::BootstrapResult::Created) => {
                tracing::info!(
                    email = %email,
                    "bootstrap instance admin created from environment"
                );
            }
            Ok(voxnexus_auth::BootstrapResult::AlreadyBootstrapped) => {
                tracing::info!("instance admin already exists; skipping bootstrap");
            }
            Err(error) => {
                tracing::error!(error = %error, "bootstrap instance admin failed");
                return Err(1);
            }
        }
    } else if config.bootstrap_admin_email.is_some() || config.bootstrap_admin_password.is_some() {
        tracing::warn!(
            "BOOTSTRAP_ADMIN_EMAIL and BOOTSTRAP_ADMIN_PASSWORD must both be set; skipping bootstrap"
        );
    }

    let registration_mode = if config.registration_open {
        RegistrationMode::Open
    } else {
        RegistrationMode::Closed
    };
    let community_creation_mode = CommunityCreationMode::parse(&config.community_creation_mode)
        .unwrap_or(CommunityCreationMode::Open);
    ensure_instance(
        &pool,
        &InstanceSeed {
            name: "VoxNexus".to_owned(),
            public_url: config.public_url.as_str().to_owned(),
            registration_mode,
            community_creation_mode,
            oidc_enabled: config.oidc_issuer.is_some(),
            oidc_issuer: config
                .oidc_issuer
                .as_ref()
                .map(|url| url.as_str().to_owned()),
            oidc_client_id: config.oidc_client_id.clone(),
        },
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "instance seed failed");
        1
    })?;

    if config.community_creation_mode_locked {
        voxnexus_auth::sync_locked_community_creation_mode(&pool, community_creation_mode)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "locked community creation mode sync failed");
                1
            })?;
        tracing::info!(
            mode = community_creation_mode.as_str(),
            "synced locked community creation mode from config"
        );
    }

    if community_creation_mode.needs_bootstrap_community(
        voxnexus_auth::count_communities(&pool)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "count communities failed");
                1
            })?,
    ) {
        if let Some(admin_id) = voxnexus_auth::first_instance_admin_id(&pool)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "lookup instance admin for community bootstrap failed");
                1
            })?
        {
            let name = config
                .bootstrap_community_name
                .clone()
                .unwrap_or_else(|| "Community".to_owned());
            match voxnexus_auth::ensure_bootstrap_community(&pool, admin_id, &name).await {
                Ok(Some(community)) => {
                    tracing::info!(
                        community_id = %community.id,
                        slug = %community.slug,
                        "bootstrap community created for single mode"
                    );
                }
                Ok(None) => {
                    tracing::info!("community already exists; skipping bootstrap community");
                }
                Err(error) => {
                    tracing::error!(error = %error, "bootstrap community failed");
                    return Err(1);
                }
            }
        } else {
            tracing::warn!(
                "community_creation_mode is single but no instance admin exists yet; \
                 set BOOTSTRAP_ADMIN_EMAIL/PASSWORD to seed the community"
            );
        }
    }

    if let Some(issuer) = config.oidc_issuer.as_ref() {
        voxnexus_auth::sync_oidc_from_config(
            &pool,
            issuer.as_str(),
            config.oidc_client_id.as_deref(),
        )
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "oidc config sync failed");
            1
        })?;
        tracing::info!(
            issuer = %issuer,
            "synced OIDC issuer and client id from config into instance settings"
        );
    }

    let redis = start_redis(config.redis_url.as_str()).await?;
    let storage = start_storage(&config).await?;
    let search = start_typesense(&config).await?;

    let job_storage = health_ping_storage(redis.clone());
    let worker = tokio::spawn(async move {
        if let Err(error) = run_health_ping_workers(job_storage, shutdown_signal()).await {
            tracing::error!(error = %error, "job workers stopped");
        }
    });
    let thumb_storage = thumbnail_storage(redis.clone());
    let thumb_worker = tokio::spawn(async move {
        if let Err(error) = run_thumbnail_stub_workers(thumb_storage, shutdown_signal()).await {
            tracing::error!(error = %error, "thumbnail workers stopped");
        }
    });

    let listener = TcpListener::bind(config.listen_addr)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, addr = %config.listen_addr, "failed to bind");
            1
        })?;
    let addr: SocketAddr = listener.local_addr().map_err(|error| {
        tracing::error!(error = %error, "failed to read listen address");
        1
    })?;
    tracing::info!(%addr, "listening");

    let app = voxnexus::http::app(voxnexus::http::AppState {
        pool,
        metrics_enabled: config.metrics_enabled,
        public_url: config.public_url.clone(),
        cookie_secure: config.cookie_secure,
        community_creation_mode_locked: config.community_creation_mode_locked,
        gateway_allow_unauth: config.gateway_allow_unauth,
        gateway_heartbeat_interval: std::time::Duration::from_millis(
            voxnexus_protocol::DEFAULT_HEARTBEAT_INTERVAL_MS,
        ),
        storage,
        redis,
        search,
        web_dist: config.web_dist.clone(),
        resume_store: std::sync::Arc::new(voxnexus_realtime::ResumeStore::new()),
        presence_hub: std::sync::Arc::new(voxnexus_realtime::PresenceHub::with_default_grace()),
        oidc_client_secret: config.oidc_client_secret.clone(),
        oidc_only: config.oidc_only,
        oidc_link_by_email: config.oidc_link_by_email,
        permission_cache: std::sync::Arc::new(PermissionCache::default()),
    });
    let serve = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "server error");
            1
        });
    worker.abort();
    thumb_worker.abort();
    serve
}

async fn start_redis(redis_url: &str) -> Result<RedisConn, i32> {
    let redis = connect(redis_url).await.map_err(|error| {
        tracing::error!(error = %error, "redis startup failed");
        1
    })?;
    ping(&redis).await.map_err(|error| {
        tracing::error!(error = %error, "redis ping failed");
        1
    })?;
    tracing::info!("redis ready");
    Ok(redis)
}

async fn start_storage(config: &Config) -> Result<Arc<dyn ObjectStore>, i32> {
    let storage = Arc::new(S3ObjectStore::new(S3ObjectStoreConfig {
        endpoint: config.s3_endpoint.as_str().to_owned(),
        access_key: config.s3_access_key.expose().to_owned(),
        secret_key: config.s3_secret_key.expose().to_owned(),
        bucket: config.s3_bucket.clone(),
        region: "us-east-1".to_owned(),
    }));
    storage.ensure_bucket().await.map_err(|error| {
        tracing::error!(error = %error, "object storage startup failed");
        1
    })?;
    tracing::info!(bucket = %config.s3_bucket, "object storage ready");
    Ok(storage)
}

async fn start_typesense(config: &Config) -> Result<Arc<dyn SearchEngine>, i32> {
    let search = Arc::new(
        TypesenseClient::new(TypesenseConfig {
            base_url: config.typesense_url.clone(),
            api_key: config.typesense_api_key.expose().to_owned(),
        })
        .map_err(|error| {
            tracing::error!(error = %error, "typesense client build failed");
            1
        })?,
    );
    search.ping().await.map_err(|error| {
        tracing::error!(error = %error, "typesense ping failed");
        1
    })?;
    search.ensure_collections().await.map_err(|error| {
        tracing::error!(error = %error, "typesense ensure collections failed");
        1
    })?;
    tracing::info!("typesense ready");
    Ok(search)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
