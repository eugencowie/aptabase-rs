use rand::Rng;
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex as SyncMutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::task::JoinHandle;

use crate::{
    config::{Config, InitOptions},
    dispatcher::EventDispatcher,
    sys::{self, SystemProperties},
};

static SESSION_TIMEOUT: Duration = Duration::from_secs(4 * 60 * 60);

fn new_session_id() -> String {
    let epoch_in_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs();

    let mut rng = rand::rng();
    let random: u64 = rng.random_range(0..=99999999);

    let id = epoch_in_seconds * 100_000_000 + random;

    id.to_string()
}

/// A tracking session.
#[derive(Debug, Clone)]
pub struct TrackingSession {
    pub id: String,
    pub last_touch_ts: OffsetDateTime,
}

impl TrackingSession {
    fn new_at(now: OffsetDateTime) -> Self {
        Self {
            id: new_session_id(),
            last_touch_ts: now,
        }
    }
}

/// The Aptabase client used to track events.
pub struct AptabaseClient {
    is_enabled: bool,
    session: SyncMutex<TrackingSession>,
    dispatcher: Arc<EventDispatcher>,
    app_version: String,
    sys_info: SystemProperties,
    flush_interval: Duration,
}

impl AptabaseClient {
    /// Creates a client using the default options.
    pub fn new(app_key: impl Into<String>, app_version: impl Into<String>) -> Self {
        Self::with_options(app_key, app_version, InitOptions::default())
    }

    /// Creates a client using custom endpoint, interval, or engine metadata options.
    pub fn with_options(
        app_key: impl Into<String>,
        app_version: impl Into<String>,
        options: InitOptions,
    ) -> Self {
        let config = Config::new(app_key.into(), &options);
        let sys_info = sys::get_info(&options);

        let is_enabled = !config.app_key.is_empty();
        let dispatcher = Arc::new(EventDispatcher::new(&config, &sys_info));

        Self {
            is_enabled,
            dispatcher,
            session: SyncMutex::new(TrackingSession::new_at(OffsetDateTime::now_utc())),
            app_version: app_version.into(),
            sys_info,
            flush_interval: config.flush_interval,
        }
    }

    /// Returns whether this client has a valid delivery configuration.
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// Starts periodic flushing on the current Tokio runtime.
    ///
    /// Abort the returned handle to stop the worker.
    pub fn start_periodic_flush(&self) -> JoinHandle<()> {
        let dispatcher = self.dispatcher.clone();
        let interval = self.flush_interval;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                dispatcher.flush().await;
            }
        })
    }

    /// Returns the current session ID, creating a new one if necessary.
    fn eval_session_id(&self) -> String {
        self.eval_session_id_at(OffsetDateTime::now_utc())
    }

    fn eval_session_id_at(&self, now: OffsetDateTime) -> String {
        let mut session = self.session.lock().expect("could not lock events");

        if (now - session.last_touch_ts) > SESSION_TIMEOUT {
            *session = TrackingSession::new_at(now);
        } else {
            session.last_touch_ts = now;
        }

        session.id.clone()
    }

    /// Enqueues an event to be sent to the server.
    pub fn track_event(&self, name: &str, props: Option<Value>) -> Result<(), String> {
        if !self.is_enabled {
            return Ok(());
        }

        if let Some(props) = &props {
            if !matches!(props, Value::Object(_)) {
                return Err(
                    "props must be `None` or the `Object` variation of `serde_json::Value`"
                        .to_owned(),
                );
            }
        }

        let ev = json!({
            "timestamp": OffsetDateTime::now_utc().format(&Rfc3339).unwrap(),
            "sessionId": self.eval_session_id(),
            "eventName": name,
            "systemProps": {
                "isDebug": self.sys_info.is_debug,
                "osName": self.sys_info.os_name,
                "osVersion": self.sys_info.os_version,
                "locale": self.sys_info.locale,
                "engineName": self.sys_info.engine_name,
                "engineVersion": self.sys_info.engine_version,
                "appVersion": self.app_version,
                "sdkVersion": concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"))
            },
            "props": props
        });

        self.dispatcher.enqueue(ev);

        Ok(())
    }

    /// Flushes the event queue.
    pub async fn flush(&self) {
        self.dispatcher.flush().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> AptabaseClient {
        AptabaseClient::new("A-US-test", "1.2.3")
    }

    #[test]
    fn serializes_required_event_fields_and_object_properties() {
        let client = client();
        client
            .track_event("settings_saved", Some(json!({ "section": "privacy" })))
            .unwrap();

        let events = client.dispatcher.queued_events();
        let event = &events[0];
        assert!(event["timestamp"].as_str().is_some());
        assert!(event["sessionId"].as_str().is_some());
        assert_eq!(event["eventName"], "settings_saved");
        assert_eq!(event["props"], json!({ "section": "privacy" }));
        assert_eq!(event["systemProps"]["appVersion"], "1.2.3");
        assert_eq!(event["systemProps"]["engineName"], "Rust");
        assert_eq!(event["systemProps"]["engineVersion"], "unknown");
        assert!(event["systemProps"]["isDebug"].is_boolean());
        assert!(event["systemProps"]["osName"].is_string());
        assert!(event["systemProps"]["osVersion"].is_string());
        assert!(event["systemProps"]["locale"].is_string());
        assert_eq!(
            event["systemProps"]["sdkVersion"],
            concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn rejects_non_object_properties_without_enqueueing() {
        let client = client();

        assert!(client
            .track_event("invalid", Some(json!([1, 2, 3])))
            .is_err());
        assert!(client.dispatcher.is_empty());
    }

    #[test]
    fn disabled_client_ignores_events() {
        let client = AptabaseClient::new("invalid", "1.0.0");

        assert!(!client.is_enabled());
        assert!(client
            .track_event("ignored", Some(json!([1, 2, 3])))
            .is_ok());
        assert!(client.dispatcher.is_empty());
    }

    #[test]
    fn reuses_session_within_four_hours() {
        let client = client();
        let now = OffsetDateTime::UNIX_EPOCH;
        *client.session.lock().unwrap() = TrackingSession {
            id: "existing".into(),
            last_touch_ts: now,
        };

        assert_eq!(
            client.eval_session_id_at(now + Duration::from_secs(60)),
            "existing"
        );
        assert_eq!(
            client.session.lock().unwrap().last_touch_ts,
            now + Duration::from_secs(60)
        );
    }

    #[test]
    fn rotates_session_after_four_hours() {
        let client = client();
        let now = OffsetDateTime::UNIX_EPOCH;
        *client.session.lock().unwrap() = TrackingSession {
            id: "expired".into(),
            last_touch_ts: now,
        };

        let session_id = client.eval_session_id_at(now + SESSION_TIMEOUT + Duration::from_secs(1));
        assert_ne!(session_id, "expired");
        assert_eq!(
            client.session.lock().unwrap().last_touch_ts,
            now + SESSION_TIMEOUT + Duration::from_secs(1)
        );
    }

    #[test]
    fn applies_engine_metadata_overrides_to_events() {
        let client = AptabaseClient::with_options(
            "A-US-test",
            "1.0.0",
            InitOptions {
                engine_name: Some("Custom Runtime".into()),
                engine_version: Some("9.1".into()),
                ..InitOptions::default()
            },
        );
        client.track_event("started", None).unwrap();

        let events = client.dispatcher.queued_events();
        assert_eq!(events[0]["systemProps"]["engineName"], "Custom Runtime");
        assert_eq!(events[0]["systemProps"]["engineVersion"], "9.1");
    }
}
