use log::error;
use std::{path::PathBuf, process::exit};

pub struct Entry {
    pub text: String,
    pub action: Box<dyn Fn() -> ()>,
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
