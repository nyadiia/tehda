use clap::Parser;
use config::Config;
use gdk::ffi::{gdk_window_set_background_rgba, GdkRGBA};
use gdk::glib::{Char, OptionArg, OptionFlags};
use gdk::EventKey;
use gio::prelude::ApplicationExt;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use gtk_sys::{gtk_style_set_background, GTK_STATE_NORMAL};
use log::error;
use log::trace;
use std::env;
use std::ffi::OsString;
use std::fs::read_dir;
use std::process::exit;

use crate::config::load_style;
use crate::modes::common::Entry;
use crate::modes::drun::get_drun_entries;
use crate::modes::run::get_run_entries;

mod config;
mod modes;

/// Wayland launcher / menu program.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to config
    #[arg(short, long)]
    config: Option<OsString>,

    /// path to css file for styling; must be valid unicode
    #[arg(short, long)]
    style: Option<String>,

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

enum Mode {
    Drun,
    Run,
    Custom(String),
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

    app.add_main_option(
        "style",
        Char::from(b's'),
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
        let mut enabled_modes: Vec<Mode> = vec![];

        // things that cause the program to exit first
        if modes.is_empty() {
            error!("No modes specified. Try `drun`!");
            exit(1);
        } else if modes.contains(&"dump_config") {
            trace!("dumping config and exiting");
            println!("{}", serde_yaml::to_string(config).unwrap());
            exit(0);
        }

        for mode in modes {
            if mode == "drun" {
                enabled_modes.push(Mode::Drun);
            } else if mode == "run" {
                enabled_modes.push(Mode::Run);
            }
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

        /*

        unsafe {
            gtk_style_set_background(, &win, GTK_STATE_NORMAL)
        }*/

        // TODO: i shouldnt have to clone that lmao
        let css_provider = load_style(args.style.clone());
        let css_context = gtk::StyleContext::new();
        css_context.add_provider(&css_provider, 1);

        gdk::Screen::default().and_then(|screen| {
            gtk::StyleContext::add_provider_for_screen(
                &screen,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            Some(())
        });

        win.connect_key_press_event(keypress_handler_with_config(config));

        gtk_layer_shell::init_for_window(&win);

        gtk_layer_shell::set_layer(&win, gtk_layer_shell::Layer::Top);

        gtk_layer_shell::set_keyboard_interactivity(&win, true);

        let outer_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer_container.set_widget_name("window");
        win.add(&outer_container);

        // set up the application's widgets
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_widget_name("inner-box");
        outer_container.add(&container);

        let input = gtk::Entry::new();
        input.set_widget_name("input");
        input.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("search"));
        container.add(&input);

        let scrolled_window =
            gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        scrolled_window.set_widget_name("entries-container");
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

        // was trying to figure out how to have Entries populate before a search
        /*
        // empty the flowbox
        flow_box
            .children()
            .into_iter()
            .for_each(|c| flow_box.remove(&c));

        let mut entries: Vec<Entry> = vec![];



        for generator in &enabled_modes
         {
            let mut new_entries = (generator)("");
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
        */

        // handle input
        input.connect_changed(move |i| {
            // empty the flowbox
            flow_box
                .children()
                .into_iter()
                .for_each(|c| flow_box.remove(&c));

            let query = i.text();

            if query.is_empty() {
                return;
            }

            let mut entries = vec![];

            for mode in &enabled_modes {
                match mode {
                    Mode::Drun => entries.append(&mut get_drun_entries(query.as_str()).collect()),
                    Mode::Run => entries.append(&mut get_run_entries(query.as_str()).collect()),
                    Mode::Custom(_) => todo!(),
                };
            }

            for entry in entries {
                let flow_box_child = gtk::FlowBoxChild::new();
                flow_box_child.style_context().add_class("flow-box-child");
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
