use std::ffi::OsString;
use std::path::Path;
use std::process::exit;
use log::{info, trace, warn};
use argh::FromArgs;
use serde::{Serialize, Deserialize};
use std::env;

#[derive(FromArgs)]
/// Wayland launcher / menu program.
struct Args {
    /// path to config
    #[argh(option, short = 'c')]
    config: Option<OsString>,

    /// print the config (usually the default one) and exit
    #[argh(switch)]
    dump_config: bool
}

fn default_modes() -> Vec<String> {
    vec!("drun".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    #[serde(default = "default_modes")]
    modes: Vec<String>
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
fn load_config(cfg_path: Option<OsString>) -> Config {
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

fn main() {
    let args: Args = argh::from_env();
    let config = load_config(args.config);

    if args.dump_config {
    println!("{}", serde_yaml::to_string(&config).unwrap());   
        exit(0);
    }

}
