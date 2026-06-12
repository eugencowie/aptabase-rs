> [!NOTE]
> This project is not affiliated with Aptabase or Sumbit Labs Ltd.

# Rust SDK for Aptabase

`aptabase-rs` is a framework-independent Rust SDK for Aptabase, an Open Source, Privacy-First, and Simple Analytics for Mobile, Desktop, and Web Apps.

## Install

Install the SDK by adding the following to your `Cargo.toml` file:

```toml
[dependencies]
aptabase-rs = "0.1.0"
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Usage

First, you need to get your `App Key` from Aptabase, you can find it in the `Instructions` menu on the left side menu.

Then create the client with your app key and application version:

```rust
use aptabase_rs::Builder;
use serde_json::json;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Builder::new(
        "<YOUR_APP_KEY>",
        env!("CARGO_PKG_VERSION"),
    )
    .build();

    client.track_event("command_started", Some(json!({ "command": "sync" })))?;
    client.flush().await;

    Ok(())
}
```

`Builder::build` constructs the client without performing network I/O or starting a background worker. Event tracking only enqueues in memory at the call site; explicitly await `flush` before a short-lived process exits.

Properties must be a `serde_json::Value::Object`. Invalid app keys disable tracking without making construction fail.

## Periodic flushing

Long-running applications can opt in to periodic flushing with `with_polling(true)`. When polling is enabled, construct the client inside an active Tokio runtime. The interval defaults to 60 seconds in release builds and 2 seconds in debug builds.

```rust
use aptabase_rs::{Builder, InitOptions};
use std::time::Duration;

let client = Builder::new("A-EU-your-app-key", "2.0.0")
    .with_options(InitOptions {
        host: None,
        flush_interval: Some(Duration::from_secs(30)),
    })
    .with_polling(true)
    .build();

client.track_event("service_started", None)?;
client.flush().await;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Events are kept only in memory, so applications should explicitly flush during orderly shutdown.

## Configuration

Use `InitOptions` for a self-hosted endpoint or for the interval used when periodic polling is enabled.

```rust
use aptabase_rs::{Builder, InitOptions};
use std::time::Duration;

let client = Builder::new("A-SH-your-app-key", "1.4.0")
    .with_options(InitOptions {
        host: Some("https://analytics.example.com".into()),
        flush_interval: Some(Duration::from_secs(30)),
    })
    .build();
```

Hosted keys beginning with `A-US-` and `A-EU-` use Aptabase's regional endpoints. `A-DEV-` targets `http://localhost:3000`, and `A-SH-` requires `host`.

## Panic hook

You can use the default panic hook to enqueue a `panic` event before the process continues to the default panic handler. If the panicking thread has an active Tokio runtime, the hook makes a best-effort delivery attempt with the same Tokio-backed transport. Without an active Tokio runtime on that thread, the panic event is only enqueued in memory before the previous panic hook runs. Applications that need predictable shutdown delivery should still call `flush().await` from normal control flow.

```rust
use aptabase_rs::Builder;

let client = Builder::new("A-EU-your-app-key", env!("CARGO_PKG_VERSION"))
    .with_default_panic_hook()
    .build();
# let _ = client;
```

For custom panic event payloads, provide your own hook:

```rust
use aptabase_rs::Builder;
use serde_json::json;

let client = Builder::new("A-EU-your-app-key", env!("CARGO_PKG_VERSION"))
    .with_panic_hook(Box::new(|client, info, message| {
        let location = info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_default();

        let _ = client.track_event(
            "panic",
            Some(json!({ "info": format!("{} ({})", message, location) })),
        );
    }))
    .build();
# let _ = client;
```

## Delivery behavior

- Requests contain at most 25 events and use Aptabase's `/api/v0/events` endpoint.
- Transport failures and HTTP 5xx responses are requeued in memory for a later flush.
- Other unsuccessful HTTP responses are discarded.
- This crate provides only the asynchronous Tokio transport; it does not expose a blocking flush API.

## Privacy

The client never tracks events automatically. Applications decide which event names and properties to send. Each event includes the app version, SDK version, debug status, OS name and version, and locale. Avoid sending personal or sensitive data in event names or properties.

See [Aptabase](https://aptabase.com) for service documentation and privacy guidance.
