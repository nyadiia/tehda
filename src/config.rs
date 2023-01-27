use std::path::Path;
use std::ffi::OsString;
use log::{info, trace, warn};
use std::env;
use serde::{Serialize, Deserialize};

fn default_modes() -> Vec<String> {
    vec!("drun".to_string())
}

fn default_width() -> i32 { 320 }
fn default_height() -> i32 { 200 }

fn default_keybind_quit() -> String { "Escape".to_string() }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Keybinds {
    #[serde(default = "default_keybind_quit")]
    pub quit: String
}

impl Default for Keybinds {
    fn default() -> Self {
        serde_yaml::from_str("{}").unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_modes")]
    pub modes: Vec<String>,

    #[serde(default = "default_width")]
    pub width: i32,

    #[serde(default = "default_height")]
    pub height: i32,

    #[serde(default)]
    pub keybinds: Keybinds
}

/// Try to load the config from a file.
fn try_load_config(path: OsString) -> Option<Config> {
    trace!("trying to load config from `{:?}`", &path);
    match std::fs::read_to_string(&path) {
        Ok(cfg_file) => match serde_yaml::from_str(&cfg_file) {
            Ok(config) => {
                trace!("loaded config from `{:?}`", path);
                Some(config)
            }
            Err(e) => {
                warn!("couldn't parse `{:?}`: {}", path, e);
                None
            }
        },
        Err(e) => {
            warn!("couldn't read `{:?}`: {}", path, e);
            None
        }
    }
}

/// Load the Tehda config.
pub fn load_config(cfg_path: Option<OsString>) -> Config {
    trace!("trying to load config");
    if let Some(p) = cfg_path {
        trace!("user provided config");
        if let Some(config) = try_load_config(p) {
            return config;
        }
    }
    // try the other default ones
    // (i wish i could put these in a separate function but rust doesn't like that)
    // also maybe TODO: gross
    vec!(
        // $XDG_CONFIG_HOME/tehda/tehda.yaml
        env::var_os("XDG_CONFIG_HOME").map(|v| Path::new(&v).join("/tehda/tehda.yaml")),
        // $HOME/.config/tehda/tehda.yaml
        env::var_os("HOME").map(|v| Path::new(&v).join("/.config/tehda/tehda.yaml")),
        Some(Path::new("~/.config/tehda/tehda.yaml").to_path_buf()),
    ).into_iter().filter_map(|p| p)
        .into_iter()
        .map(OsString::from)
        .find_map(try_load_config)
        // <autumn>: i'm just gonna .unwrap() this 'cause i don't think it'll 
        //           ever fail
        // <ash>: famous last words
        //
        // if we can't find any configs that we can read and parse,
        // just load the default config
        .unwrap_or_else(|| serde_yaml::from_str("{}").unwrap())
}