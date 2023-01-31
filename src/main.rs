use clap::Parser;
use config::Config;
use gdk::glib::{Char, OptionArg, OptionFlags};
use gdk::EventKey;
use gio::prelude::ApplicationExt;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use log::error;
use log::trace;
use std::ffi::OsString;
use std::process::exit;

use crate::modes::common::Entry;
use crate::modes::drun::get_drun_entries;

mod config;
mod modes;

/// Wayland launcher / menu program.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to config
    #[arg(short, long)]
    config: Option<OsString>,

    /// which modes to use, comma-separated
    /// built-in modes are:
    ///  - drun (run desktop apps)
    ///  - run (run something on PATH)
    ///  - dump_config (dump config and quit)
    #[arg(short, long, verbatim_doc_comment)]
    modes: String,
}

fn keypress_handler_with_config(
    // <autumn>: if i change [this type] to a ref it gets mad about me moving the value
    //           in there and then has a whole bunch of lifetime bullshit going on
    // <ash>: THIS is the point of leaking! autumn im gonna leak just for you
    config: &Config,
) -> impl Fn(&ApplicationWindow, &EventKey) -> Inhibit + '_ {
    |_, keypress| {
        match keypress.keyval().name() {
            Some(s) => {
                if &s == &config.keybinds.quit {
                    exit(0)
                }

                // TODO: add more, maybe make this into a HashMap if it gets large
            }
            None => {}
        }

        gtk::Inhibit(false)
    }
}

fn main() {
    pretty_env_logger::init();
    trace!("starting tehda");
    // safety: i dont care about the leaking here. we're using the config  and
    // args immutably for the program's duration, so ill be damned if i dont
    // have a &'static
    let args = Box::leak(Box::new(Args::parse()));
    let config = Box::leak(Box::new(config::load_config(args.config.as_ref())));

    let app = Application::builder()
        .application_id("page.mikufan.tehda")
        .build();

    // gtk my behated
    app.add_main_option(
        "modes",
        Char::from(b'm'),
        OptionFlags::NONE,
        OptionArg::String,
        "",
        None,
    );

    app.add_main_option(
        "config",
        Char::from(b'c'),
        OptionFlags::NONE,
        OptionArg::String,
        "",
        None,
    );

    app.connect_activate(|app| {
        // TODO: it would be cool if i could do this outside of this block
        // since thats where it makes sense
        // but rust
        let modes: Vec<&str> = args.modes.split(",").collect();

        if modes.contains(&"dump_config") {
            trace!("dumping config and exiting");
            println!("{}", serde_yaml::to_string(config).unwrap());
            exit(0);
        }

        if modes.is_empty() {
            error!("No modes specified. Try `drun`!");
            exit(1);
        }

        let mut modes_generators: Vec<Box<dyn Fn(&str) -> Vec<Entry>>> = vec![];

        if modes.contains(&"drun") {
            modes_generators.push(Box::new(get_drun_entries));
        }

        trace!("building window");
        // TODO: this works, but gtk starts spewing `CRITICAL`s into stdout
        let win = ApplicationWindow::builder()
            .application(app)
            .default_width(config.width)
            .default_height(config.height)
            .title("tehda")
            .window_position(gtk::WindowPosition::None)
            .gravity(gdk::Gravity::Center)
            .decorated(false)
            .resizable(false)
            .has_focus(true)
            // TODO: we probably don't actually need all events
            .events(gdk::EventMask::ALL_EVENTS_MASK)
            .build();

        win.connect_key_press_event(keypress_handler_with_config(config));
        win.set_border_width(12);

        gtk_layer_shell::init_for_window(&win);

        gtk_layer_shell::set_layer(&win, gtk_layer_shell::Layer::Top);

        gtk_layer_shell::set_keyboard_interactivity(&win, true);

        // set up the application's widgets
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        win.add(&container);

        let input = gtk::Entry::new();
        input.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("search"));
        container.add(&input);

        let scrolled_window =
            gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        scrolled_window.set_vexpand(true);
        container.add(&scrolled_window);

        let viewport = gtk::Viewport::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        scrolled_window.add(&viewport);

        let inner_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        viewport.add(&inner_box);

        let flow_box = gtk::FlowBox::new();
        flow_box.set_orientation(gtk::Orientation::Horizontal);
        flow_box.set_max_children_per_line(1);
        inner_box.add(&flow_box);

        // handle input
        input.connect_changed(move |i| {
            // empty the flowbox
            flow_box
                .children()
                .into_iter()
                .for_each(|c| flow_box.remove(&c));

            let query = i.text();

            let mut entries: Vec<Entry> = vec![];

            for generator in &modes_generators {
                let mut new_entries = (generator)(query.as_str());
                entries.append(&mut new_entries);
            }

            for entry in entries {
                let flow_box_child = gtk::FlowBoxChild::new();
                let label = gtk::Label::new(Some(entry.text.as_str()));
                flow_box_child.add(&label);
                flow_box.add(&flow_box_child);
                flow_box_child.show();
                label.show();

                flow_box_child.connect_activate(move |_| (entry.action)());
            }
        });

        trace!("showing window");
        win.show_all();
    });

    app.run();
}
