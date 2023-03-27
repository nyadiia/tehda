use gtk::prelude::*;

use std::process::exit;

use crate::modes::common::{Entry, Mode};
use crate::modes::drun::get_drun_entries;
use crate::modes::run::get_run_entries;

use std::time::Instant;

fn make_entry(flow_box: &gtk::FlowBox, entry: Entry) {
    let flow_box_child = gtk::FlowBoxChild::new();
    flow_box_child.style_context().add_class("flow-box-child");
    flow_box.add(&flow_box_child);

    if let Some(actions) = entry.alternate_actions {
        let expander = gtk::Expander::new(Some(entry.text.as_str()));
        flow_box_child.add(&expander);
        expander.show();

        let flow_box = gtk::FlowBox::new();
        flow_box.set_orientation(gtk::Orientation::Vertical);
        flow_box.set_max_children_per_line(1);
        expander.add(&flow_box);
        flow_box.show();

        for action in actions {
            let flow_box_child = gtk::FlowBoxChild::new();

            flow_box_child.style_context().add_class("flow-box-child");

            let l = gtk::Label::new(Some(action.0.as_str()));
            l.show();
            flow_box_child.add(&l);
            flow_box.add(&flow_box_child);
            flow_box_child.show();
            flow_box_child.connect_activate(move |_| {
                (action.1)();
                exit(0);
            });
            l.show();
        }
    } else {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.show();
        flow_box_child.add(&container);

        let label = gtk::Label::new(Some(entry.text.as_str()));
        container.add(&label);
        label.show();
    }

    flow_box_child.show();

    flow_box_child.connect_activate(move |_| (entry.action)());
}

#[allow(clippy::module_name_repetitions)] // `handle` would be a weird name
pub fn handle_input(i: &gtk::Entry, flow_box: &gtk::FlowBox, modes: &Vec<Mode>) {
    // empty the flowbox
    flow_box
        .children()
        .into_iter()
        .for_each(|c| flow_box.remove(&c));

    // get query
    let query = i.text();
    if query.is_empty() {
        return;
    }

    // get the entries to render
    let mut entries = vec![];

    let now = Instant::now();
    for mode in modes {
        match mode {
            Mode::Drun => entries.append(&mut get_drun_entries(query.as_str()).collect()),
            Mode::Run => entries.append(&mut get_run_entries(query.as_str()).collect()),
            Mode::Custom(_a) => todo!(),
        };
    }
    let elapsed_time = now.elapsed();
    log::debug!(
        "collecting entries took {:.4}ms",
        elapsed_time.as_secs_f64() * 1000f64
    );

    // let open_entries = Rc::new(RefCell::new(HashSet::new()));

    for entry in entries {
        make_entry(flow_box, entry);
    }
}
