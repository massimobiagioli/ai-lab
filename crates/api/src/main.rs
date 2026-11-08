//! AI-Lab `api` binary.
//!
//! Serves the WP-16 edge router; WP-03 delivers the health probe and WP-05 the
//! startup exposure and master-key guard.
#![forbid(unsafe_code)]

use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let listen = std::env::var("AILAB_API_LISTEN").unwrap_or_else(|_| {
        let bind = std::env::var("AILAB_BIND").unwrap_or_else(|_| "127.0.0.1".to_owned());
        format!("{bind}:3000")
    });
    let bind = listen
        .rsplit_once(':')
        .map_or_else(|| listen.clone(), |(host, _)| host.to_owned());
    let token = std::env::var("AILAB_API_TOKEN").ok();
    let master_key = std::env::var("AILAB_MASTER_KEY").ok();

    if args.iter().any(|arg| arg == "check-exposure") {
        if let Err(error) = validate_startup(&bind, token.as_deref(), master_key.as_deref()) {
            eprintln!("refusing to start: {error}");
            std::process::exit(78);
        }
        println!("exposure: OK");
        return;
    }

    if args.iter().any(|arg| arg == "healthcheck") {
        let healthy = ailab_infrastructure::health::check_urls(
            &["AILAB_DATABASE_URL", "AILAB_REDIS_URL"],
            Duration::from_secs(3),
        );
        std::process::exit(i32::from(!healthy));
    }

    if let Err(error) = ailab_application::Config::from_env() {
        eprintln!("refusing to start: {error}");
        std::process::exit(78);
    }

    if let Err(error) = validate_startup(&bind, token.as_deref(), master_key.as_deref()) {
        eprintln!("refusing to start: {error}");
        std::process::exit(78);
    }

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("api: cannot start the async runtime: {error}");
            std::process::exit(1);
        }
    };
    runtime.block_on(async move {
        let app = ailab_api::server::router(ailab_api::server::AppState::new());
        let listener = match tokio::net::TcpListener::bind(&listen).await {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("api: cannot bind {listen}: {error}");
                std::process::exit(1);
            }
        };
        if let Err(error) = axum::serve(listener, app).await {
            eprintln!("api: server error: {error}");
            std::process::exit(1);
        }
    });
}

fn validate_startup(
    bind: &str,
    token: Option<&str>,
    master_key: Option<&str>,
) -> Result<(), String> {
    ailab_infrastructure::exposure::validate_exposure(bind, token)
        .map_err(|error| error.to_string())?;
    if let Some(key) = master_key.filter(|value| !value.trim().is_empty()) {
        ailab_infrastructure::crypto::MasterKey::from_base64(key, 1)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
