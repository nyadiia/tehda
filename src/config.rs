use log::{trace, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

/// Returns the default object as parsed by Serde.
/// Requires all fields on the object to either 1. have a #[serde(default)] or
/// 2. be `Optional<T>`.
fn make_serde_default<'a, T: Deserialize<'a>>() -> T {
    // <autumn>: i'm just gonna .unwrap() this 'cause i don't think it'll
    //           ever fail
    // <ash>: famous last words
    serde_yaml::from_str("{}").unwrap()
}

fn default_width() -> i32 {
    800
}
fn default_height() -> i32 {
    450
}

fn default_keybind_quit() -> String {
    "Escape".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Keybinds {
    #[serde(default = "default_keybind_quit")]
    pub quit: String,
}

impl Default for Keybinds {
    fn default() -> Self {
        make_serde_default()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_width")]
    pub width: i32,

    #[serde(default = "default_height")]
    pub height: i32,

    #[serde(default)]
    pub keybinds: Keybinds,
}

/// Try to load the config from a file.
fn try_load_config<P: AsRef<Path>>(path: P) -> Option<Config> {
    trace!("trying to load config from `{:?}`", path.as_ref());

    match std::fs::read_to_string(&path) {
        Ok(cfg_file) => match serde_yaml::from_str(&cfg_file) {
            Ok(config) => {
                trace!("loaded config from `{:?}`", path.as_ref());
                Some(config)
            }
            Err(e) => {
                warn!("couldn't parse `{:?}`: {}", path.as_ref(), e);
                None
            }
        },
        Err(e) => {
            warn!("couldn't read `{:?}`: {}", path.as_ref(), e);
            None
        }
    }
}

/// Returns the path to attempt to load the config from
fn config_path() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME").map(|v| {
                let mut buf = PathBuf::from(v);
                buf.push(".config");
                buf
            })
        })
        .map(|mut v| {
            v.push("tehda/tehda.yaml");
            v
        })
}

/// Load the Tehda config.
pub fn load_config<P: AsRef<Path>>(cfg_path: Option<P>) -> Config {
    trace!("trying to load config");
    if let Some(p) = cfg_path {
        trace!("user provided config");
        if let Some(config) = try_load_config(p) {
            return config;
        }
    }

    config_path()
        .and_then(try_load_config)
        // if we can't find any configs that we can read and parse,
        // just load the default config
        .unwrap_or_else(make_serde_default)
}
