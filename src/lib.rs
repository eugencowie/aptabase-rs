mod builder;
mod client;
mod config;
mod dispatcher;
mod sys;

use std::{panic::PanicHookInfo, sync::Arc, time::Duration};

pub use client::{new_session_id, AptabaseClient};
use config::Config;

#[derive(Default, Debug, Clone)]
pub struct InitOptions {
    pub host: Option<String>,
    pub flush_interval: Option<Duration>,
}

/// The Aptabase client builder
pub struct Builder {
    app_key: String,
    app_version: String,
    session_id: Option<String>,
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
            session_id: None,
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

    /// Sets a custom panic hook.
    pub fn with_panic_hook(mut self, hook: PanicHook) -> Self {
        self.panic_hook = Some(hook);
        self
    }

    /// Builds and initializes the client
    pub fn build(self) -> Arc<AptabaseClient> {
        let cfg = Config::new(self.app_key, self.session_id, self.options);
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
