mod client;
mod config;
mod dispatcher;
mod sys;

use std::{panic::PanicHookInfo, sync::Arc, time::Duration};

pub use client::AptabaseClient;
use config::Config;
use serde_json::json;

#[derive(Default, Debug, Clone)]
pub struct InitOptions {
    pub host: Option<String>,
    pub flush_interval: Option<Duration>,
}

/// The Aptabase client builder
pub struct Builder {
    app_key: String,
    app_version: String,
    enable_polling: bool,
    panic_hook: Option<PanicHook>,
    options: InitOptions,
}

pub type PanicHook =
    Box<dyn Fn(&AptabaseClient, &PanicHookInfo<'_>, String) + 'static + Sync + Send>;

fn get_panic_message(info: &PanicHookInfo) -> String {
    let payload = info.payload();
    if let Some(s) = payload.downcast_ref::<&str>() {
        return s.to_string();
    } else if let Some(s) = payload.downcast_ref::<String>() {
        return s.to_string();
    }

    format!("{:?}", payload)
}

impl Builder {
    /// Creates a new builder.
    pub fn new(app_key: &str, app_version: &str) -> Self {
        Self {
            app_key: app_key.into(),
            app_version: app_version.into(),
            enable_polling: false,
            panic_hook: None,
            options: Default::default(),
        }
    }

    /// Sets custom options to use for the Aptabase client.
    pub fn with_options(mut self, opts: InitOptions) -> Self {
        self.options = opts;
        self
    }

    /// Sets whether polling should be enabled.
    pub fn with_polling(mut self, enable: bool) -> Self {
        self.enable_polling = enable;
        self
    }

    /// Sets a custom panic hook.
    pub fn with_panic_hook(mut self, hook: PanicHook) -> Self {
        self.panic_hook = Some(hook);
        self
    }

    /// Enables the default panic hook.
    pub fn with_default_panic_hook(self) -> Self {
        self.with_panic_hook(Box::new(|client, info, msg| {
            let location = info
                .location()
                .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
                .unwrap_or_else(|| "".to_string());
            let _ = client.track_event(
                "panic",
                Some(json!({
                    "info": format!("{} ({})", msg, location),
                })),
            );
        }))
    }

    /// Builds and initializes the client
    pub fn build(self) -> Arc<AptabaseClient> {
        let cfg = Config::new(self.app_key, self.options);
        let client = Arc::new(AptabaseClient::new(&cfg, self.app_version));

        if self.enable_polling {
            client.start_polling(cfg.flush_interval);
        }

        if let Some(hook) = self.panic_hook {
            let default_panic = std::panic::take_hook();
            let hook_client = client.clone();
            std::panic::set_hook(Box::new(move |info| {
                let msg = get_panic_message(info);
                hook(&hook_client, info, msg);
                hook_client.flush_blocking();
                default_panic(info);
            }));
        }

        client
    }
}
