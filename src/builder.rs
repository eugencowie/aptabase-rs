// Fork-authored additions to `Builder`, kept out of `lib.rs` so that `lib.rs`
// stays line-parallel to upstream's and merges cleanly.

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
