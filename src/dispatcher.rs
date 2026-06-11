use std::{
    cmp::min,
    collections::VecDeque,
    sync::{Arc, RwLock},
    time::Duration,
};

use log::{debug, trace};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Url,
};
use serde_json::{json, Value};

use crate::{config::Config, sys::SystemProperties};

static HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) struct EventDispatcher {
    url: Url,
    queue: Arc<RwLock<VecDeque<Value>>>,
    http_client: reqwest::Client,
}

impl EventDispatcher {
    pub fn new(config: &Config, sys: &SystemProperties) -> Self {
        let mut headers = HeaderMap::new();
        let app_key_header = HeaderValue::from_str(config.app_key.as_str())
            .expect("failed to define App Key header value");
        headers.insert("App-Key", app_key_header);
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let user_agent = format!(
            "{}/{} {}/{} {}",
            sys.os_name, sys.os_version, sys.engine_name, sys.engine_version, sys.locale
        );
        let http_client = reqwest::Client::builder()
            .timeout(HTTP_REQUEST_TIMEOUT)
            .default_headers(headers)
            .user_agent(user_agent)
            .build()
            .expect("could not build http client");

        let queue = Arc::new(RwLock::new(VecDeque::new()));

        Self {
            url: config.ingest_api_url.clone(),
            queue,
            http_client,
        }
    }

    pub fn is_empty(&self) -> bool {
        let queue = self.queue.read().expect("could not lock queue for reading");
        queue.is_empty()
    }

    pub fn enqueue(&self, event: Value) {
        let mut queue = self.queue.write().expect("could not lock queue");
        queue.push_back(event);
    }

    pub fn enqueue_many(&self, events: Vec<Value>) {
        let mut queue = self.queue.write().expect("could not lock queue");
        queue.extend(events);
    }

    #[cfg(test)]
    pub(crate) fn queued_events(&self) -> Vec<Value> {
        self.queue
            .read()
            .expect("could not lock queue for reading")
            .iter()
            .cloned()
            .collect()
    }

    fn dequeue_many(&self, max: usize) -> Vec<Value> {
        let mut queue = self.queue.write().expect("could not lock queue");
        if queue.is_empty() {
            return Vec::new();
        }

        let dequeue_len = min(queue.len(), max);
        queue.drain(..dequeue_len).collect()
    }

    pub async fn flush(&self) {
        trace!("flushing tracking events");
        if self.is_empty() {
            trace!("nothing to send");
            return;
        }

        let mut failed_items = Vec::new();
        loop {
            let events_to_send = self.dequeue_many(25);
            if events_to_send.is_empty() {
                break;
            }

            trace!("preparing {} events to send", events_to_send.len());

            let body = json!(events_to_send);
            let response = self
                .http_client
                .post(self.url.clone())
                .json(&body)
                .send()
                .await;
            match response {
                Ok(response) => match response.status().is_success() {
                    true => {
                        trace!("sent {} tracking events", events_to_send.len());
                    }
                    false => {
                        debug!(
                            "failed to track_event with status code {}",
                            response.status()
                        );
                        if response.status().is_server_error() {
                            failed_items.extend(events_to_send);
                        }
                    }
                },
                Err(err) => {
                    failed_items.extend(events_to_send);
                    debug!("failed to track_event: {}", err);
                }
            }
        }

        self.enqueue_many(failed_items);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::InitOptions, sys};
    use serde_json::json;
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
    };

    #[derive(Debug)]
    struct RecordedRequest {
        request_line: String,
        headers: Vec<(String, String)>,
        body: Value,
    }

    fn dispatcher(url: Url) -> EventDispatcher {
        let config = Config {
            app_key: "A-DEV-test".into(),
            ingest_api_url: url,
            flush_interval: Duration::from_secs(1),
        };
        EventDispatcher::new(&config, &sys::get_info(&InitOptions::default()))
    }

    fn read_request(stream: &mut TcpStream) -> RecordedRequest {
        let mut bytes = Vec::new();
        let mut buffer = [0; 4096];
        let header_end = loop {
            let count = stream.read(&mut buffer).unwrap();
            assert!(
                count > 0,
                "connection closed before request headers arrived"
            );
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                break index + 4;
            }
        };

        let headers_text = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
        let mut lines = headers_text.split("\r\n");
        let request_line = lines.next().unwrap().to_string();
        let headers = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_string()))
            .collect::<Vec<_>>();
        let content_length = headers
            .iter()
            .find(|(name, _)| name == "content-length")
            .and_then(|(_, value)| value.parse::<usize>().ok())
            .unwrap_or(0);

        while bytes.len() - header_end < content_length {
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0, "connection closed before request body arrived");
            bytes.extend_from_slice(&buffer[..count]);
        }

        RecordedRequest {
            request_line,
            headers,
            body: serde_json::from_slice(&bytes[header_end..header_end + content_length]).unwrap(),
        }
    }

    fn spawn_server(statuses: Vec<u16>) -> (Url, mpsc::Receiver<RecordedRequest>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            for status in statuses {
                let (mut stream, _) = listener.accept().unwrap();
                let request = read_request(&mut stream);
                sender.send(request).unwrap();
                let reason = if status == 200 {
                    "OK"
                } else if status == 400 {
                    "Bad Request"
                } else {
                    "Internal Server Error"
                };
                write!(
                    stream,
                    "HTTP/1.1 {status} {reason}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
                .unwrap();
            }
        });

        (
            format!("http://{address}/api/v0/events").parse().unwrap(),
            receiver,
        )
    }

    #[tokio::test]
    async fn empty_flush_makes_no_request() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/api/v0/events", listener.local_addr().unwrap())
            .parse()
            .unwrap();
        let dispatcher = dispatcher(url);

        dispatcher.flush().await;
        assert!(dispatcher.is_empty());
    }

    #[tokio::test]
    async fn sends_headers_endpoint_and_batches_of_at_most_25() {
        let (url, requests) = spawn_server(vec![200, 200]);
        let dispatcher = dispatcher(url);
        for index in 0..26 {
            dispatcher.enqueue(json!({ "index": index }));
        }

        dispatcher.flush().await;

        let first = requests.recv().unwrap();
        let second = requests.recv().unwrap();
        assert_eq!(first.request_line, "POST /api/v0/events HTTP/1.1");
        assert_eq!(first.body.as_array().unwrap().len(), 25);
        assert_eq!(second.body.as_array().unwrap().len(), 1);
        assert!(first
            .headers
            .contains(&("app-key".into(), "A-DEV-test".into())));
        assert!(first
            .headers
            .contains(&("content-type".into(), "application/json".into())));
        assert!(first
            .headers
            .iter()
            .any(|(name, value)| { name == "user-agent" && value.contains("Rust/unknown") }));
        assert!(dispatcher.is_empty());
    }

    #[tokio::test]
    async fn requeues_transport_failures() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let dispatcher = dispatcher(format!("http://{address}/api/v0/events").parse().unwrap());
        dispatcher.enqueue(json!({ "event": 1 }));

        dispatcher.flush().await;

        assert_eq!(dispatcher.queued_events().len(), 1);
    }

    #[tokio::test]
    async fn requeues_server_errors() {
        let (url, requests) = spawn_server(vec![500]);
        let dispatcher = dispatcher(url);
        dispatcher.enqueue(json!({ "event": 1 }));

        dispatcher.flush().await;

        requests.recv().unwrap();
        assert_eq!(dispatcher.queued_events().len(), 1);
    }

    #[tokio::test]
    async fn discards_non_server_http_errors() {
        let (url, requests) = spawn_server(vec![400]);
        let dispatcher = dispatcher(url);
        dispatcher.enqueue(json!({ "event": 1 }));

        dispatcher.flush().await;

        requests.recv().unwrap();
        assert!(dispatcher.is_empty());
    }
}
