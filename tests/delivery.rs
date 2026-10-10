// Delivery behaviour, exercised through the public interface against a local
// stand-in for the Aptabase server. An `SH` App Key sends to whichever host
// the test supplies.

use std::{net::TcpListener, sync::Arc, time::Duration};

use aptabase_rs::{AptabaseClient, Builder, InitOptions};
use serde_json::{json, Value};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, Request, ResponseTemplate,
};

const SH_APP_KEY: &str = "A-SH-1234567890";
const APP_VERSION: &str = "1.2.3";

fn client_for(host: &str) -> Arc<AptabaseClient> {
    Builder::new(SH_APP_KEY, APP_VERSION)
        .with_options(InitOptions {
            host: Some(host.to_owned()),
            flush_interval: None,
        })
        .build()
}

fn events_endpoint(status: u16) -> Mock {
    Mock::given(method("POST"))
        .and(path("/api/v0/events"))
        .respond_with(ResponseTemplate::new(status))
}

fn batches(requests: &[Request]) -> Vec<Vec<Value>> {
    requests
        .iter()
        .map(|request| request.body_json().unwrap())
        .collect()
}

fn batch_sizes(requests: &[Request]) -> Vec<usize> {
    batches(requests).iter().map(Vec::len).collect()
}

fn event_names(requests: &[Request]) -> Vec<String> {
    let mut names: Vec<String> = batches(requests)
        .into_iter()
        .flatten()
        .map(|event| event["eventName"].as_str().unwrap().to_owned())
        .collect();
    names.sort();
    names
}

fn sorted_keys(value: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort();
    keys
}

#[tokio::test]
async fn flush_sends_events_in_batches_of_25() {
    // Arrange
    let server = MockServer::start().await;
    let delivered = events_endpoint(200).mount_as_scoped(&server).await;
    let client = client_for(&server.uri());
    for i in 0..60 {
        client.track_event(&format!("event_{i}"), None).unwrap();
    }

    // Act
    client.flush().await;

    // Assert
    assert_eq!(
        batch_sizes(&delivered.received_requests().await),
        [25, 25, 10]
    );
}

#[tokio::test]
async fn server_error_keeps_events_for_the_next_flush() {
    // Arrange
    let server = MockServer::start().await;
    let failed = events_endpoint(503)
        .up_to_n_times(1)
        .mount_as_scoped(&server)
        .await;
    let delivered = events_endpoint(200).mount_as_scoped(&server).await;
    let client = client_for(&server.uri());
    client.track_event("app_started", None).unwrap();
    client.track_event("app_exited", None).unwrap();

    // Act
    client.flush().await;
    client.flush().await;

    // Assert
    assert_eq!(
        event_names(&failed.received_requests().await),
        ["app_exited", "app_started"]
    );
    assert_eq!(
        event_names(&delivered.received_requests().await),
        ["app_exited", "app_started"]
    );
}

#[tokio::test]
async fn transport_failure_keeps_events_for_the_next_flush() {
    // Arrange
    let addr = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let client = client_for(&format!("http://{addr}"));
    client.track_event("app_started", None).unwrap();

    // Act
    // Nothing listens on the port during the first flush
    client.flush().await;
    let server = MockServer::builder()
        .listener(TcpListener::bind(addr).unwrap())
        .start()
        .await;
    let delivered = events_endpoint(200).mount_as_scoped(&server).await;
    client.flush().await;

    // Assert
    assert_eq!(
        event_names(&delivered.received_requests().await),
        ["app_started"]
    );
}

#[tokio::test]
async fn client_error_drops_events() {
    // Arrange
    let server = MockServer::start().await;
    let rejected = events_endpoint(400)
        .up_to_n_times(1)
        .mount_as_scoped(&server)
        .await;
    let delivered = events_endpoint(200).mount_as_scoped(&server).await;
    let client = client_for(&server.uri());
    client.track_event("app_started", None).unwrap();

    // Act
    client.flush().await;

    // Assert
    assert_eq!(
        event_names(&rejected.received_requests().await),
        ["app_started"]
    );

    // Act
    client.flush().await;

    // Assert
    assert_eq!(
        event_names(&delivered.received_requests().await),
        Vec::<String>::new()
    );
}

#[tokio::test]
async fn events_are_sent_in_aptabase_format() {
    // Arrange
    let server = MockServer::start().await;
    let delivered = events_endpoint(200).mount_as_scoped(&server).await;
    let client = client_for(&server.uri());
    let props = json!({ "plan": "pro", "seats": 3 });
    client
        .track_event("app_started", Some(props.clone()))
        .unwrap();

    // Act
    client.flush().await;

    // Assert
    let requests = delivered.received_requests().await;
    let request = &requests[0];
    assert_eq!(request.headers["App-Key"], SH_APP_KEY);
    assert_eq!(request.headers["Content-Type"], "application/json");

    let event = &batches(&requests)[0][0];
    assert_eq!(
        sorted_keys(event),
        [
            "eventName",
            "props",
            "sessionId",
            "systemProps",
            "timestamp"
        ]
    );
    assert_eq!(event["eventName"], "app_started");
    assert_eq!(event["props"], props);
    assert!(!event["sessionId"].as_str().unwrap().is_empty());
    OffsetDateTime::parse(event["timestamp"].as_str().unwrap(), &Rfc3339).unwrap();

    let system = &event["systemProps"];
    assert_eq!(
        sorted_keys(system),
        [
            "appVersion",
            "isDebug",
            "locale",
            "osName",
            "osVersion",
            "sdkVersion"
        ]
    );
    assert_eq!(system["appVersion"], APP_VERSION);
    assert_eq!(
        system["sdkVersion"],
        concat!("aptabase-rs@", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(system["isDebug"], cfg!(debug_assertions));
    assert!(system["osName"].is_string());
    assert!(system["osVersion"].is_string());
    assert!(system["locale"].is_string());
}

#[tokio::test]
async fn invalid_app_key_sends_nothing() {
    // Arrange
    let server = MockServer::start().await;
    let delivered = events_endpoint(200).mount_as_scoped(&server).await;
    let client = Builder::new("A-XX-1234567890", APP_VERSION)
        .with_options(InitOptions {
            host: Some(server.uri()),
            flush_interval: None,
        })
        .build();
    client.track_event("app_started", None).unwrap();

    // Act
    client.flush().await;

    // Assert
    assert_eq!(
        event_names(&delivered.received_requests().await),
        Vec::<String>::new()
    );
}

#[tokio::test]
async fn polling_flushes_without_a_manual_flush() {
    // Arrange
    let server = MockServer::start().await;
    let delivered = events_endpoint(200).mount_as_scoped(&server).await;
    let client = Builder::new(SH_APP_KEY, APP_VERSION)
        .with_polling(true)
        .with_options(InitOptions {
            host: Some(server.uri()),
            flush_interval: Some(Duration::from_millis(50)),
        })
        .build();

    // Act
    client.track_event("app_started", None).unwrap();

    // Assert
    let arrived = tokio::time::timeout(Duration::from_secs(5), async {
        while delivered.received_requests().await.is_empty() {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await;
    assert!(arrived.is_ok(), "no flush within 5s");
    assert_eq!(
        event_names(&delivered.received_requests().await),
        ["app_started"]
    );
}
