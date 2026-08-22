use std::net::SocketAddr;

use tokio::net::TcpListener;
use voxnexus_config::Config;

#[tokio::main]
async fn main() {
    if let Err(code) = run().await {
        std::process::exit(code);
    }
}

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
    });
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "server error");
            1
        })?;
    Ok(())
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
