use std::env;
use std::path::PathBuf;

fn get(var: &'static str) -> Option<PathBuf> {
    env::var_os(var).map(PathBuf::from)
}

pub fn cache_dir() -> Option<PathBuf> {
    let def = || dirs::home_dir().map(|home| home.join(".cache"));
    get("XDG_CACHE_HOME").or_else(def)
}

pub fn config_dir() -> Option<PathBuf> {
    let def = || dirs::home_dir().map(|home| home.join(".config"));
    get("XDG_CONFIG_HOME").or_else(def)
}

pub fn data_dir() -> Option<PathBuf> {
    let def = || dirs::home_dir().map(|home| home.join(".local").join("share"));
    get("XDG_DATA_HOME").or_else(def)
}

pub fn state_dir() -> Option<PathBuf> {
    let def = || dirs::home_dir().map(|home| home.join(".local").join("state"));
    get("XDG_STATE_HOME").or_else(def)
}

/// Returns "$HOME/.local.dir".
/// See https://specifications.freedesktop.org/basedir-spec/latest/#variables
pub fn executable_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".local").join("bin"))
}
