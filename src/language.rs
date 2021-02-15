use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

pub fn get_config_path() -> PathBuf {
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

pub fn get_words(lang: String) -> std::io::Result<Vec<String>> {
    let lang_dir = get_config_path().join("language");
    let lang_file = lang_dir.join(lang);

    let buf = fs::File::open(lang_file)?;
    Ok(io::BufReader::new(buf)
        .lines()
        .filter_map(|t| t.ok())
        .collect())
}
