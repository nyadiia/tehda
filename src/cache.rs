use std::{env, path::PathBuf};

use crate::modes::common::Entry;

fn cache_folder_path() -> Option<PathBuf> {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME").map(|v| {
                let mut buf = PathBuf::from(v);
                buf.push(".cache");
                buf
            })
        })
        .map(|v| v.join("tehda"))
}

pub fn add_entry_to_cache(entry: Entry) {}

pub fn get_cache_entries() -> Vec<Entry> {
    cache_folder_path().map(|path| {});
    todo!()
}
