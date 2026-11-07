//! Health probes shared by the runtime services.
//!
//! The probes are dependency-free on purpose: they are the last line of the
//! container health model and must work even when the application has not
//! finished booting.

use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Parse `scheme://[user:pass@]host[:port][/path]` into a host and port.
///
/// Defaults: postgres 5432, redis 6379, tcp 2375.
#[must_use]
pub fn endpoint(url: &str) -> Option<(String, u16)> {
    let without_scheme = url.split("://").nth(1).unwrap_or(url);
    let authority = without_scheme.split('/').next()?;
    let host_port = authority.rsplit('@').next()?;
    let default_port = if url.starts_with("postgres") {
        5432
    } else if url.starts_with("redis") {
        6379
    } else {
        2375
    };
    if let Some((host, port)) = host_port.rsplit_once(':')
        && let Ok(parsed) = port.parse::<u16>()
    {
        return Some((host.trim_matches(['[', ']']).to_owned(), parsed));
    }
    Some((host_port.trim_matches(['[', ']']).to_owned(), default_port))
}

/// True when a TCP connection to the URL host and port succeeds within timeout.
#[must_use]
pub fn tcp_reachable(url: &str, timeout: Duration) -> bool {
    let Some((host, port)) = endpoint(url) else {
        return false;
    };
    let Ok(addresses) = (host.as_str(), port).to_socket_addrs() else {
        return false;
    };
    addresses
        .into_iter()
        .any(|address| TcpStream::connect_timeout(&address, timeout).is_ok())
}

/// True when the environment variable exists and is not blank.
#[must_use]
pub fn require_env(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| !value.trim().is_empty())
}

/// True when every URL in `vars` is set and reachable.
#[must_use]
pub fn check_urls(vars: &[&str], timeout: Duration) -> bool {
    vars.iter()
        .all(|name| std::env::var(name).is_ok_and(|url| tcp_reachable(&url, timeout)))
}

#[cfg(test)]
mod tests {
    use super::{endpoint, require_env};

    #[test]
    fn parses_postgres_url() {
        assert_eq!(
            endpoint("postgres://user:pass@postgres:5432/ailab"),
            Some(("postgres".to_owned(), 5432))
        );
    }

    #[test]
    fn parses_redis_url_with_default_port() {
        assert_eq!(
            endpoint("redis://redis/0"),
            Some(("redis".to_owned(), 6379))
        );
    }

    #[test]
    fn parses_tcp_proxy_url() {
        assert_eq!(
            endpoint("tcp://docker-socket-proxy:2375"),
            Some(("docker-socket-proxy".to_owned(), 2375))
        );
    }

    #[test]
    fn missing_env_is_not_required() {
        assert!(!require_env("AILAB_TEST_VAR_THAT_DOES_NOT_EXIST"));
    }
}
