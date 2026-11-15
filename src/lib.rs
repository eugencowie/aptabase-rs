mod builder_ext;
mod client;
mod config;
mod dispatcher;
mod errors;
mod sys;
#[cfg(test)]
mod test_server;

use std::{
    panic::PanicHookInfo,
    sync::Arc,
    time::{Duration, Instant},
};

pub use client::{session::new_session_id, AptabaseClient};
use config::Config;
use errors::ErrorKind;
pub use errors::ErrorReport;

#[derive(Default, Debug, Clone)]
pub struct InitOptions {
    pub host: Option<String>,
    pub flush_interval: Option<Duration>,
}

/// How long a crash waits for its report to be sent before the app exits.
const CRASH_FLUSH_TIMEOUT: Duration = Duration::from_secs(3);

/// The Aptabase client builder
pub struct Builder {
    app_key: String,
    app_version: String,
    session_id: Option<String>,
    enable_polling: bool,
    panic_hook: Option<PanicHook>,
    crash_reporting: bool,
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
            crash_reporting: false,
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

    /// Reports crashes and uncaught errors to Aptabase, on the Errors page of the dashboard.
    ///
    /// - A panic on the main thread ends the app: it is reported as a fatal crash, and the app waits up to three
    ///   seconds for the report to be sent before it exits.
    /// - A panic in a task spawned on a Tokio runtime ends only that task, and one on a thread of your own ends only
    ///   that thread: both are reported as errors.
    ///
    /// A hook set with [`Builder::with_panic_hook`] still runs, before the panic is reported.
    pub fn with_crash_reporting(mut self) -> Self {
        self.crash_reporting = true;
        self
    }

    /// Builds and initializes the client
    pub fn build(self) -> Arc<AptabaseClient> {
        let crash_reporting = self.crash_reporting;
        let cfg = Config::new(self.app_key, self.options);
        let client = Arc::new(AptabaseClient::new(&cfg, self.app_version));
        client.seed_session_id(self.session_id);

        if self.enable_polling {
            client.start_polling(cfg.flush_interval);
        }

        if self.panic_hook.is_some() || crash_reporting {
            let custom_hook = self.panic_hook;
            let default_panic = std::panic::take_hook();
            let hook_client = client.clone();
            std::panic::set_hook(Box::new(move |info| {
                let msg = get_panic_message(info);
                if let Some(hook) = &custom_hook {
                    hook(&hook_client, info, msg.clone());
                }
                // One budget for everything the hook sends, so the app never waits longer than that in all.
                let deadline = Instant::now() + CRASH_FLUSH_TIMEOUT;
                if crash_reporting {
                    let (report, kind) = errors::panic_report(info, &msg);
                    if kind == ErrorKind::Crash {
                        hook_client.report_crash(report, CRASH_FLUSH_TIMEOUT);
                    } else {
                        hook_client.track_error_as(report, kind);
                    }
                }
                // What the custom hook tracked, after the crash's report so that nothing holds it up.
                let left = deadline.saturating_duration_since(Instant::now());
                if custom_hook.is_some() && !left.is_zero() {
                    hook_client.flush_within(left);
                }
                default_panic(info);
            }));
        }

        client
    }
}
