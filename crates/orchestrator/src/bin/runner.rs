//! AI-Lab `runner` binary.
//!
//! The sandbox control plane lands with WP-14. WP-03 delivers the platform
//! health probe and a minimal readiness listener so the worker can verify that
//! at least one runner is reachable (WP-03 §3.6/§3.7).
#![forbid(unsafe_code)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

fn main() {
    if std::env::args().any(|arg| arg == "healthcheck") {
        let healthy = ailab_infrastructure::health::require_env("AILAB_RUNNER_TOKEN")
            && ailab_infrastructure::health::check_urls(&["DOCKER_HOST"], Duration::from_secs(3));
        std::process::exit(i32::from(!healthy));
    }

    let bind = std::env::var("AILAB_RUNNER_BIND").unwrap_or_else(|_| "0.0.0.0:8081".to_owned());
    let listener = match TcpListener::bind(&bind) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("runner: cannot bind {bind}: {error}");
            std::process::exit(1);
        }
    };
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else {
            continue;
        };
        let mut buffer = [0_u8; 512];
        let _ = stream.read(&mut buffer);
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
        );
    }
}
