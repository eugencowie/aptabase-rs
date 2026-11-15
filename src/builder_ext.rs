// Fork-only additions to `Builder`, kept separate to minimise the diff against
// upstream's `lib.rs`.

use serde_json::json;

use crate::Builder;

impl Builder {
    /// Sets the initial session ID.
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Sets whether polling should be enabled.
    pub fn with_polling(mut self, enable: bool) -> Self {
        self.enable_polling = enable;
        self
    }

    /// Enables the default panic hook.
    pub fn with_default_panic_hook(self) -> Self {
        self.with_panic_hook(Box::new(|client, info, message| {
            let location = info
                .location()
                .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
                .unwrap_or_default();
            let _ = client.track_event(
                "panic",
                Some(json!({
                    "info": format!("{} ({})", message, location),
                })),
            );
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::{
        test_server::{runtime, server},
        Builder, InitOptions,
    };

    #[test]
    fn with_polling_flushes_without_a_manual_flush() {
        let server = server(&[200]);
        let _runtime = runtime().enter();
        let client = Builder::new("A-SH-0000000000", "1.4.0")
            .with_options(InitOptions {
                host: Some(server.base.clone()),
                flush_interval: Some(Duration::from_millis(50)),
            })
            .with_polling(true)
            .build();
        client.track_event("app_started", None).unwrap();
        let started = Instant::now();
        while server.requests.lock().unwrap().is_empty()
            && started.elapsed() < Duration::from_secs(5)
        {
            std::thread::sleep(Duration::from_millis(20));
        }
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].1[0]["eventName"], "app_started");
    }
}
