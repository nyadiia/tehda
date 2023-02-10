use std::{path::PathBuf, process::exit};

use gdk::{glib::GString, prelude::AppInfoExt};
use gio;
use log::error;

use super::common::Entry;

/// run the executable at the given path
fn run_executable(path: PathBuf) {
    match subprocess::Exec::cmd(path).detached().join() {
        Ok(_) => exit(0),
        Err(e) => {
            error!("error running command: {}", e);
            exit(1);
        }
    }
}

/// convert a filename to desktop app info
fn filename_to_info(filename: GString) -> Option<gio::DesktopAppInfo> {
    // TODO: we might be able to memoize this or something in order to speed
    // this up; that's just a premature optimization thought though
    gio::DesktopAppInfo::new(filename.as_str())
}

/// convert desktop app info to an Entry
fn info_to_entry(info: gio::DesktopAppInfo) -> Entry {
    Entry {
        text: info.display_name().to_string(),
        action: Box::new(move || run_executable(info.executable())),
    }
}

// TODO: basically return what get_drun_entries would if search was empty
// putting Entries in alphabetical and then putting last used at the top
/*
pub fn init_drun_entries() -> Vec<Entry> {
    gio::DesktopAppInfo::generic_name(&self)
}
*/
/// gets Entries for desktop apps available on the system (and known to gio)
pub fn get_drun_entries(query: &str) -> Vec<Entry> {
    // it isn't clear from the docs, so i'll describe it here: the search
    // returns a vector of ranks, each rank being a vector of filenames
    // each rank is unsorted within itself, it just means that each
    // filename in that search has the same relatedness to the query
    // as each other
    //
    // TODO: it could pay off to sort within the ranks ourselves
    gio::DesktopAppInfo::search(query)
        .into_iter()
        .flatten()
        .map(filename_to_info)
        .filter_map(|i| i.map(info_to_entry))
        .collect()
}
