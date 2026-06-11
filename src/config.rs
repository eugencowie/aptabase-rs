use std::time::Duration;

use log::debug;
use reqwest::Url;

/// Optional Aptabase client configuration.
#[derive(Default, Debug, Clone)]
pub struct InitOptions {
    /// Base URL for an `A-SH-*` self-hosted app key.
    pub host: Option<String>,
    /// Interval used by [`crate::AptabaseClient::start_periodic_flush`].
    pub flush_interval: Option<Duration>,
    /// Value reported as `engineName` in event system properties.
    pub engine_name: Option<String>,
    /// Value reported as `engineVersion` in event system properties.
    pub engine_version: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) app_key: String,
    pub(crate) ingest_api_url: Url,
    pub(crate) flush_interval: Duration,
}

static LOCAL: &str = "http://localhost:3000";
static US_REGION: &str = "https://us.aptabase.com";
static EU_REGION: &str = "https://eu.aptabase.com";

#[cfg(not(debug_assertions))]
static DEFAULT_FLUSH_INTERVAL: Duration = Duration::from_secs(60);

#[cfg(debug_assertions)]
static DEFAULT_FLUSH_INTERVAL: Duration = Duration::from_secs(2);

const VALID_REGIONS: &[&str] = &["US", "EU", "DEV", "SH"];

impl Config {
    pub(crate) fn new(app_key: String, opts: &InitOptions) -> Self {
        let parts = app_key.split('-').collect::<Vec<&str>>();
        if parts.len() != 3
            || parts[0] != "A"
            || parts[2].is_empty()
            || !VALID_REGIONS.contains(&parts[1])
        {
            debug!(
                "The Aptabase App Key '{}' is invalid. Tracking will be disabled.",
                app_key
            );
            return Config::default();
        }

        let base_url: String = match parts[1] {
            "EU" => EU_REGION.into(),
            "US" => US_REGION.into(),
            "DEV" => LOCAL.into(),
            "SH" => {
                if let Some(host) = &opts.host {
                    host.clone()
                } else {
                    debug!("Host parameter must be defined when using Self-Hosted App Key. Tracking will be disabled.");
                    return Config::default();
                }
            }
            _ => return Config::default(),
        };

        let Ok(ingest_api_url) =
            format!("{}/api/v0/events", base_url.trim_end_matches('/')).parse()
        else {
            debug!(
                "The Aptabase host '{}' is invalid. Tracking will be disabled.",
                base_url
            );
            return Config::default();
        };

        Self {
            app_key,
            ingest_api_url,
            flush_interval: opts.flush_interval.unwrap_or(DEFAULT_FLUSH_INTERVAL),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_key: String::new(),
            ingest_api_url: Url::parse(LOCAL).unwrap(),
            flush_interval: DEFAULT_FLUSH_INTERVAL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> InitOptions {
        InitOptions {
            flush_interval: Some(Duration::from_secs(30)),
            ..InitOptions::default()
        }
    }

    #[test]
    fn configures_hosted_and_development_regions() {
        let us = Config::new("A-US-123".into(), &options());
        let eu = Config::new("A-EU-123".into(), &options());
        let dev = Config::new("A-DEV-123".into(), &options());

        assert_eq!(
            us.ingest_api_url.as_str(),
            "https://us.aptabase.com/api/v0/events"
        );
        assert_eq!(
            eu.ingest_api_url.as_str(),
            "https://eu.aptabase.com/api/v0/events"
        );
        assert_eq!(
            dev.ingest_api_url.as_str(),
            "http://localhost:3000/api/v0/events"
        );
        assert_eq!(us.flush_interval, Duration::from_secs(30));
    }

    #[test]
    fn configures_self_hosted_region() {
        let config = Config::new(
            "A-SH-123".into(),
            &InitOptions {
                host: Some("https://analytics.example.com/".into()),
                ..InitOptions::default()
            },
        );

        assert_eq!(
            config.ingest_api_url.as_str(),
            "https://analytics.example.com/api/v0/events"
        );
    }

    #[test]
    fn invalid_configuration_disables_tracking() {
        for app_key in ["", "invalid", "B-US-123", "A-US-", "A-XX-123"] {
            assert!(Config::new(app_key.into(), &InitOptions::default())
                .app_key
                .is_empty());
        }

        assert!(Config::new("A-SH-123".into(), &InitOptions::default())
            .app_key
            .is_empty());
        assert!(Config::new(
            "A-SH-123".into(),
            &InitOptions {
                host: Some("not a url".into()),
                ..InitOptions::default()
            }
        )
        .app_key
        .is_empty());
    }
}
