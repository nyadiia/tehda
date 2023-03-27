use gtk::{traits::CssProviderExt, CssProvider};
use log::{trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Debug,
    path::{Path, PathBuf},
};

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

fn default_keybind_open_alternate_actions() -> String {
    "Right".to_string()
}

fn default_keybind_quit() -> String {
    "Escape".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Keybinds {
    #[serde(default = "default_keybind_quit")]
    pub quit: String,

    #[serde(default = "default_keybind_open_alternate_actions")]
    pub open_alternate_actions: String,
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

impl Config {
    /// Serializes the config to a string.
    ///
    /// This is required due to `lazy_static` wrapping the type, causing it
    /// to not be `Serialize` when we need to serialize it in `main`.
    pub fn serialize(&self) -> String {
        // safety: this is safe to unwrap, since the config must be valid
        // for it to exist
        serde_yaml::to_string(self).unwrap()
    }
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

fn config_folder_path() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME").map(|v| {
                let mut buf = PathBuf::from(v);
                buf.push(".config");
                buf
            })
        })
        .map(|v| v.join("tehda"))
}

fn config_file_path(path: &str) -> Option<PathBuf> {
    config_folder_path().map(|v| v.join(path))
}

fn style_path() -> Option<PathBuf> {
    config_file_path("tehda.css")
}

/// Returns the path to attempt to load the config from
fn config_path() -> Option<PathBuf> {
    config_file_path("tehda.yaml")
}

const DEFAULT_CSS: &[u8] = b"
* {
    background-color: transparent;
}
#window {
    background-color: #000000;
    color: #ffffff;
    border-radius: 1rem;
}
#inner-box {
    margin: 1rem;
}
";

pub fn load_style(path: &Option<String>) -> CssProvider {
    trace!("trying to load styles");
    if let Some(p) = path {
        trace!("user provided style");
        let provider = CssProvider::new();
        match provider.load_from_path(p.as_str()) {
            Ok(_) => return provider,
            Err(e) => warn!("error loading styles: {e}"),
        };
    }

    style_path()
        .and_then(|p| {
            let provider = CssProvider::new();
            // TODO: gross, i hate the borrow checker
            match provider.load_from_path(
                p.as_os_str()
                    .to_str()
                    .expect("gtk-rs requires a valid unicode path to the css file"),
            ) {
                Ok(_) => Some(provider),
                Err(e) => {
                    warn!("error loading styles: {e}");
                    None
                }
            }
        })
        .unwrap_or_else(|| {
            trace!("loading default styles");
            let provider = CssProvider::new();
            match provider.load_from_data(DEFAULT_CSS) {
                Ok(_) => provider,
                Err(e) => {
                    panic!("error loading default styles: {e}")
                }
            }
        })
}

/// Load the Tehda config.
#[allow(clippy::module_name_repetitions)] // this is a better name
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
