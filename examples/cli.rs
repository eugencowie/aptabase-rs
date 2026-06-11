use aptabase_rs::AptabaseClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_key = std::env::var("APTABASE_APP_KEY").unwrap_or_default();
    let client = AptabaseClient::new(app_key, env!("CARGO_PKG_VERSION"));

    client.track_event("cli_started", None)?;
    client.flush().await;

    Ok(())
}
