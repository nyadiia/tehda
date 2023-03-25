use std::{collections::HashMap, process::exit};

use gdk::{glib::GString, prelude::AppInfoExt};
use gio;

use super::common::{run_executable, ActionFn, Entry};

/// convert a filename to desktop app info
fn filename_to_info(filename: GString) -> Option<gio::DesktopAppInfo> {
    // TODO: we might be able to memoize this or something in order to speed
    // this up; that's just a premature optimization thought though
    gio::DesktopAppInfo::new(filename.as_str())
}
/*
fn info_to_alternate_actions(info: gio::DesktopAppInfo) -> Option<HashMap<String, ActionFn>> {
    let alternate_action_names = info.list_actions();
    if (alternate_action_names.is_empty()) {
        None
    } else {
        let actions_with_display_names = alternate_action_names.iter().map(|name| {
            let s = name.as_str();
            (s, info.action_name(s))
        });
        let mut actions: HashMap<String, ActionFn> = HashMap::new();
        for (action_name, display_name) in actions_with_display_names {
            actions.insert(
                display_name.to_string(),
                Box::new(|| {
                    &info.launch_action(action_name, gio::AppLaunchContext::NONE);
                    exit(0);
                }),
            );
        }
        Some(actions)
    }
}*/

/// convert desktop app info to an Entry
fn info_to_entry(info: gio::DesktopAppInfo) -> Entry {
    Entry {
        text: info.display_name().to_string(),
        action: Box::new(move || run_executable(info.executable())),
        alternate_actions: None,
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
pub fn get_drun_entries(query: &str) -> impl Iterator<Item = Entry> + '_ {
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
}
