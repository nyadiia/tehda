use super::common::{run_executable, Entry};
use lazy_static::lazy_static;
use std::{env, fs::read_dir, path::PathBuf};

fn get_path_values() -> Vec<(String, PathBuf)> {
    env::var("PATH")
        .ok()
        .map(|path| {
            path.split(":")
                .map(|dir_path| read_dir(dir_path).ok())
                .flatten() // vec<directories> -> directories
                .flatten() // directories -> files
                .filter_map(|f| f.ok())
                .map(|dir_entry| {
                    (
                        dir_entry.file_name().to_string_lossy().to_string(),
                        dir_entry.path(),
                    )
                })
                .collect()
        })
        .unwrap_or(vec![])
}

lazy_static! {
    static ref PATH_ENTRIES: Vec<(String, PathBuf)> = get_path_values();
}

pub fn get_run_entries(query: &str) -> Vec<Entry> {
    PATH_ENTRIES
        .iter()
        .filter(|entry| entry.0.starts_with(query))
        .map(|f| Entry {
            text: f.0.to_string(),
            action: Box::new(move || run_executable(f.1.clone())),
        })
        .collect()
}
