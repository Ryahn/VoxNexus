use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use voxnexus_config::REQUIRED_KEYS;
use voxnexus_db::{test_database_url, TEST_DATABASE_URL_ENV};

fn sample_env() -> Vec<(&'static str, String)> {
    REQUIRED_KEYS
        .iter()
        .map(|key| {
            let value = match *key {
                "DATABASE_URL" => "postgres://voxnexus:voxnexus@127.0.0.1:5432/voxnexus",
                "REDIS_URL" => "redis://127.0.0.1:6379",
                "S3_ENDPOINT" => "http://127.0.0.1:8333",
                "S3_ACCESS_KEY" | "S3_SECRET_KEY" => "any",
                "S3_BUCKET" => "voxnexus",
                "TYPESENSE_URL" => "http://127.0.0.1:8108",
                "TYPESENSE_API_KEY" => "ts",
                "PUBLIC_URL" => "http://127.0.0.1:8080",
                _ => unreachable!(),
            };
            (*key, value.to_string())
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
    let deadline = Instant::now() + Duration::from_secs(15);
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
