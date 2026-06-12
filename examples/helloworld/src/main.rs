// Copy .env.example to .env and set APTABASE_KEY.

use aptabase_rs::Builder;
use dotenvy_macro::dotenv;
use serde_json::json;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Builder::new(dotenv!("APTABASE_KEY"), env!("CARGO_PKG_VERSION"))
        .with_default_panic_hook()
        .with_polling(true) // long-running processes can use polling to send events at regular intervals
        .build();

    client.track_event(
        "cli_started",
        Some(json!({
            "msg": "Hello, world!"
        })),
    )?;

    client.flush().await;
    Ok(())
}
