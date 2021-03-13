use std::env;
use std::path::{Path, PathBuf};

pub fn get_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    let os_conf_path = env::var_os("HOME").map(|s| Path::new(&s).join(".config"));
    #[cfg(target_os = "windows")]
    let os_conf_path = env::var_os("APPDATA").map(|s| Path::new(s));
    #[cfg(target_os = "macos")]
    let os_conf_path = env::var_os("HOME").map(|s| Path::new(&s).join(".config"));

    os_conf_path
        .unwrap_or(env::current_dir().unwrap())
        .join("ttyper")
}
