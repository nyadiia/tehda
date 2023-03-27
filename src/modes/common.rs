use log::error;
use std::{collections::HashMap, path::PathBuf, process::exit};

pub type ActionFn = Box<dyn Fn()>;

pub struct Entry {
    pub text: String,
    pub action: ActionFn,
    pub alternate_actions: Option<HashMap<String, ActionFn>>,
    pub open: bool,
}

/// run the executable at the given path
pub fn run_executable(path: PathBuf) {
    match subprocess::Exec::cmd(path).detached().join() {
        Ok(_) => exit(0),
        Err(e) => {
            error!("error running command: {}", e);
            exit(1);
        }
    }
}
