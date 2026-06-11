# aptabase-rs

`aptabase-rs` is a framework-independent Aptabase client for Rust applications. It synchronously queues events in memory and delivers them with an asynchronous Tokio/Reqwest transport.

## Installation

```toml
[dependencies]
aptabase-rs = "1.0.0"
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## CLI usage

Create the client with an Aptabase app key and your application version. Event tracking does not perform network I/O; explicitly await `flush` before a short-lived process exits.

```rust
use aptabase_rs::AptabaseClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AptabaseClient::new("A-US-your-app-key", env!("CARGO_PKG_VERSION"));

    client.track_event("command_started", Some(json!({ "command": "sync" })))?;
    client.flush().await;

    Ok(())
}
```

Properties must be a `serde_json::Value::Object`. Invalid app keys disable tracking without making construction fail.

## Periodic flushing

Long-running applications can start a periodic worker on an active Tokio runtime. The interval defaults to 60 seconds in release builds and 2 seconds in debug builds.

```rust
use aptabase_rs::AptabaseClient;

let client = AptabaseClient::new("A-EU-your-app-key", "2.0.0");
let worker = client.start_periodic_flush();

client.track_event("service_started", None)?;

// Stop the worker during shutdown, then deliver anything still queued.
worker.abort();
client.flush().await;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Start at most one periodic worker per client. Events are kept only in memory, so applications should explicitly flush during orderly shutdown.

## Configuration

Use `InitOptions` for a self-hosted endpoint, a custom periodic interval, or runtime metadata overrides.

```rust
use aptabase_rs::{AptabaseClient, InitOptions};
use std::time::Duration;

let client = AptabaseClient::with_options(
    "A-SH-your-app-key",
    "1.4.0",
    InitOptions {
        host: Some("https://analytics.example.com".into()),
        flush_interval: Some(Duration::from_secs(30)),
        engine_name: Some("my-cli-runtime".into()),
        engine_version: Some("1.4.0".into()),
    },
);
```

Hosted keys beginning with `A-US-` and `A-EU-` use Aptabase's regional endpoints. `A-DEV-` targets `http://localhost:3000`, and `A-SH-` requires `host`.

## Delivery behavior

- Requests contain at most 25 events and use Aptabase's `/api/v0/events` endpoint.
- Transport failures and HTTP 5xx responses are requeued in memory for a later flush.
- Other unsuccessful HTTP responses are discarded.
- This crate provides only the asynchronous Tokio transport; it does not provide a blocking client.

## Privacy

The client never tracks events automatically. Applications decide which event names and properties to send. Each event includes the app version, SDK version, debug status, OS name and version, locale, and configured engine name and version. Avoid sending personal or sensitive data in event names or properties.

See [Aptabase](https://aptabase.com) for service documentation and privacy guidance.
