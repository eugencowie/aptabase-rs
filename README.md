> [!NOTE]
> This project is not affiliated with Aptabase or Sumbit Labs Ltd.

# Rust SDK for Aptabase

`aptabase-rs` is a framework-independent Rust SDK for Aptabase, an Open Source, Privacy-First, and Simple Analytics for Mobile, Desktop, and Web Apps.

## Install

Install the SDK by adding the following to your `Cargo.toml` file:

`Cargo.toml`

```toml
[dependencies]
aptabase-rs = "0.1.0"
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Usage

First, you need to get your `App Key` from Aptabase, you can find it in the `Instructions` menu on the left side menu.

Then create the client with your app key and application version:

`src/main.rs`

```rust
use aptabase_rs::Builder;

let client = Builder::new(
    "<YOUR_APP_KEY>", // 👈 this is where you enter your App Key
    env!("CARGO_PKG_VERSION"),
)
.build();
```

You can then start sending events from Rust by calling the `track_event` method on `client`.

As an example, you can add `app_started` and `app_exited` events like this:

```rust
use aptabase_rs::Builder;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Builder::new("<YOUR_APP_KEY>", env!("CARGO_PKG_VERSION")).build();
    client.track_event("app_started", None);

    // do some work here...
    
    client.track_event("app_exited", None);
    client.flush().await;
    Ok(())
}
```

A few important notes:

1. The SDK will automatically enhance the event with some useful information, like the OS, the app version, and other things.
2. You're in control of what gets sent to Aptabase. This SDK does not automatically track any events, you need to call `track_event` manually.
    - Because of this, it's generally recommended to at least track an event at startup.
3. You do not need to await for the `track_event` function, it'll run in the background.
4. Only strings and numbers values are allowed on custom properties.

## Providing the APTABASE_KEY via .env

It's possible to load the APTABASE_KEY from a .env file at compile time using the `dotenvy_macro` crate. The `.env` file needs to be
in the project directory for the `dotevny_macro` crate to find it properly.

Add the `use` declaration to where you are building the SDK (likely `main.rs`), and then call it where you would put the key.

```rust
use aptabase_rs::Builder;
use dotenvy_macro::dotenv;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Builder::new(dotenv!("APTABASE_KEY"), env!("CARGO_PKG_VERSION")).build();
    client.track_event("app_started", None);

    // do some work here...
    
    client.track_event("app_exited", None);
    client.flush().await;
    Ok(())
}
```

## Periodic flushing

Calling `track_event` only enqueues events to be sent to the server, you need to explicitly await `flush` to actually send the queued events. For short-lived applications, this would typically be done at the end of the application's lifecycle.

Long-running applications can opt in to periodic flushing with `with_polling(true)`. The interval defaults to 60 seconds in release builds and 2 seconds in debug builds. This can be customized with `with_options(InitOptions)`. It is important that you still flush manually before the applcation exits, so that any remaining events in the queue are sent to the server.

```rust
use aptabase_rs::{Builder, InitOptions};
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Builder::new("<YOUR_APP_KEY>", env!("CARGO_PKG_VERSION"))
        .with_polling(true)
        .with_options(InitOptions {
            flush_interval: Some(Duration::from_secs(30)),
            host: None,
        })
        .build();

    // do some long-running work here...

    client.flush().await;
    Ok(())
}
```

## Panic hook

You can use the default panic hook to enqueue a `panic` event before the process continues to the default panic handler. If the panicking thread has an active Tokio runtime, the hook makes a best-effort delivery attempt with the same Tokio-backed transport. Without an active Tokio runtime on that thread, the panic event is only enqueued in memory before the previous panic hook runs.

```rust
use aptabase_rs::Builder;

let client = Builder::new("<YOUR_APP_KEY>", env!("CARGO_PKG_VERSION"))
    .with_default_panic_hook()
    .build();
```

For custom panic event payloads, provide your own hook:

```rust
use aptabase_rs::Builder;
use serde_json::json;

let client = Builder::new("<YOUR_APP_KEY>", env!("CARGO_PKG_VERSION"))
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
```

## Delivery behavior

- Requests contain at most 25 events and use Aptabase's `/api/v0/events` endpoint.
- Transport failures and HTTP 5xx responses are requeued in memory for a later flush.
- Other unsuccessful HTTP responses are discarded.
- This crate provides only the asynchronous Tokio transport; it does not expose a blocking flush API.

## Privacy

The client never tracks events automatically. Applications decide which event names and properties to send. Each event includes the app version, SDK version, debug status, OS name and version, and locale. Avoid sending personal or sensitive data in event names or properties.

See [Aptabase](https://aptabase.com) for service documentation and privacy guidance.
