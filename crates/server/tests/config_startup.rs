use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use voxnexus_config::REQUIRED_KEYS;
use voxnexus_db::{test_database_url, TEST_DATABASE_URL_ENV};

fn sample_env() -> Vec<(&'static str, String)> {
    let redis_url = std::env::var("REDIS_URL_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "redis://127.0.0.1:6379".to_owned());
    let s3_endpoint = std::env::var("S3_ENDPOINT_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8333".to_owned());
    let s3_access = std::env::var("S3_ACCESS_KEY_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "any".to_owned());
    let s3_secret = std::env::var("S3_SECRET_KEY_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "minioadmin".to_owned());
    let typesense_url = std::env::var("TYPESENSE_URL_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8108".to_owned());
    let typesense_key = std::env::var("TYPESENSE_API_KEY_TEST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "ts".to_owned());

    REQUIRED_KEYS
        .iter()
        .map(|key| {
            let value = match *key {
                "DATABASE_URL" => "postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus".to_owned(),
                "REDIS_URL" => redis_url.clone(),
                "S3_ENDPOINT" => s3_endpoint.clone(),
                "S3_ACCESS_KEY" => s3_access.clone(),
                "S3_SECRET_KEY" => s3_secret.clone(),
                "S3_BUCKET" => "voxnexus".to_owned(),
                "TYPESENSE_URL" => typesense_url.clone(),
                "TYPESENSE_API_KEY" => typesense_key.clone(),
                "PUBLIC_URL" => "http://127.0.0.1:8080".to_owned(),
                _ => unreachable!(),
            };
            (*key, value)
        })
        .collect()
}

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_voxnexus"));
    command.env_remove("VOXNEXUS_CONFIG");
    command.env_remove(TEST_DATABASE_URL_ENV);
    command.env_remove("LISTEN_ADDR");
    command.env_remove("METRICS_ENABLED");
    command.env_remove("LOG_FORMAT");
    for key in REQUIRED_KEYS {
        command.env_remove(*key);
    }
    command
}

struct KillOnDrop(Child);

impl KillOnDrop {
    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.0.try_wait()
    }
}

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn missing_database_url_exits_non_zero() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut command = command();
    command.current_dir(dir.path());
    for (key, value) in sample_env() {
        if key != "DATABASE_URL" {
            command.env(key, value);
        }
    }

    let output = command.output().expect("run voxnexus");
    assert!(
        !output.status.success(),
        "expected failure, got status {:?}, stderr {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("DATABASE_URL"),
        "stderr should name DATABASE_URL, got: {stderr}"
    );
}

#[test]
fn server_serves_health_when_test_database_is_set() {
    let Some(database_url) = test_database_url() else {
        eprintln!("skipping: set {TEST_DATABASE_URL_ENV} to exercise listening server");
        return;
    };
    if let Err(reason) = startup_deps_reachable() {
        eprintln!("skipping full server startup: {reason}");
        return;
    }

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        listener.local_addr().expect("addr").port()
    };
    let listen = format!("127.0.0.1:{port}");

    let dir = tempfile::tempdir().expect("tempdir");
    let mut command = command();
    command.current_dir(dir.path());
    command.env("LISTEN_ADDR", &listen);
    command.env("LOG_FORMAT", "json");
    for (key, value) in sample_env() {
        if key == "DATABASE_URL" {
            command.env(key, &database_url);
        } else {
            command.env(key, value);
        }
    }

    let mut child = KillOnDrop(command.spawn().expect("spawn voxnexus"));
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut last_error = String::new();
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("wait") {
            drop(child);
            panic!("server exited before becoming healthy ({status}): {last_error}");
        }
        match http_get(&listen, "/health") {
            Ok((status, request_id, body)) => {
                assert_eq!(status, 200, "body={body}");
                assert!(body.contains("ok"), "{body}");
                assert!(
                    uuid::Uuid::parse_str(&request_id).is_ok(),
                    "request id {request_id}"
                );
                drop(child);
                return;
            }
            Err(error) => {
                last_error = error;
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
    drop(child);
    panic!("server never became healthy: {last_error}");
}

fn startup_deps_reachable() -> Result<(), String> {
    let env = sample_env();
    let lookup = |key: &str| {
        env.iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.as_str())
            .expect("required key")
    };
    tcp_connect_url(lookup("REDIS_URL"), "Redis")?;
    tcp_connect_url(lookup("S3_ENDPOINT"), "S3")?;
    tcp_connect_url(lookup("TYPESENSE_URL"), "Typesense")?;
    Ok(())
}

fn tcp_connect_url(url: &str, label: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|error| format!("{label} URL: {error}"))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| format!("{label} URL missing host"))?;
    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| format!("{label} URL missing port"))?;
    TcpStream::connect_timeout(
        &format!("{host}:{port}")
            .parse()
            .map_err(|error| format!("{label} addr: {error}"))?,
        Duration::from_secs(2),
    )
    .map_err(|error| format!("{label} at {host}:{port} not reachable ({error})"))?;
    Ok(())
}

fn http_get(addr: &str, path: &str) -> Result<(u16, String, String), String> {
    let mut stream =
        TcpStream::connect(addr).map_err(|error| format!("connect {addr}: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| error.to_string())?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .map_err(|error| error.to_string())?;
    let mut buf = String::new();
    stream
        .read_to_string(&mut buf)
        .map_err(|error| error.to_string())?;
    let mut lines = buf.split("\r\n");
    let status_line = lines.next().ok_or_else(|| buf.clone())?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| buf.clone())?
        .parse::<u16>()
        .map_err(|error| error.to_string())?;
    let mut request_id = String::new();
    for line in lines.by_ref() {
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("x-request-id: ") {
            request_id = value.to_string();
        }
        if let Some(value) = line.strip_prefix("X-Request-Id: ") {
            request_id = value.to_string();
        }
    }
    let body = lines.collect::<Vec<_>>().join("\r\n");
    Ok((status, request_id, body))
}
