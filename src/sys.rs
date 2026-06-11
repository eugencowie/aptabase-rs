use crate::config::InitOptions;

static DEFAULT_ENGINE_NAME: &str = "Rust";
static DEFAULT_ENGINE_VERSION: &str = "unknown";

#[cfg(debug_assertions)]
static IS_DEBUG: bool = true;

#[cfg(not(debug_assertions))]
static IS_DEBUG: bool = false;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SystemProperties {
    pub is_debug: bool,
    pub os_name: String,
    pub os_version: String,
    pub locale: String,
    pub engine_name: String,
    pub engine_version: String,
}

#[cfg(target_os = "linux")]
fn is_flatpak() -> bool {
    use std::env::var;
    var("FLATPAK_ID").is_ok()
        || var("container")
            .map(|x| x.to_lowercase().trim() == "flatpak")
            .unwrap_or(false)
}

pub(crate) fn get_info(options: &InitOptions) -> SystemProperties {
    let info = os_info::get();
    let locale = sys_locale::get_locale().unwrap_or_default();

    let os_name = match info.os_type() {
        os_info::Type::Macos => "macOS".to_string(),
        os_info::Type::Windows => "Windows".to_string(),
        #[cfg(target_os = "linux")]
        _ if is_flatpak() => "Flatpak".to_string(),
        _ => info.os_type().to_string(),
    };

    SystemProperties {
        is_debug: IS_DEBUG,
        os_name,
        os_version: info.version().to_string(),
        locale,
        engine_name: options
            .engine_name
            .clone()
            .unwrap_or_else(|| DEFAULT_ENGINE_NAME.to_string()),
        engine_version: options
            .engine_version
            .clone()
            .unwrap_or_else(|| DEFAULT_ENGINE_VERSION.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_generic_engine_defaults() {
        let info = get_info(&InitOptions::default());

        assert_eq!(info.engine_name, "Rust");
        assert_eq!(info.engine_version, "unknown");
    }

    #[test]
    fn applies_engine_overrides() {
        let info = get_info(&InitOptions {
            engine_name: Some("My Runtime".into()),
            engine_version: Some("2.4.0".into()),
            ..InitOptions::default()
        });

        assert_eq!(info.engine_name, "My Runtime");
        assert_eq!(info.engine_version, "2.4.0");
    }
}
