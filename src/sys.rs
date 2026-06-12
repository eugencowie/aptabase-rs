#[cfg(debug_assertions)]
static IS_DEBUG: bool = true;

#[cfg(not(debug_assertions))]
static IS_DEBUG: bool = false;

pub struct SystemProperties {
    pub is_debug: bool,
    pub os_name: String,
    pub os_version: String,
    pub locale: String,
}

#[cfg(target_os = "linux")]
fn is_flatpak() -> bool {
    use std::env::var;
    var("FLATPAK_ID").is_ok()
        || var("container")
            .map(|x| x.to_lowercase().trim() == "flatpak")
            .unwrap_or(false)
}

pub fn get_info() -> SystemProperties {
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
    }
}
