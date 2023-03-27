use clap::Parser;
use config::Config;
use gio::prelude::ApplicationExt;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use log::error;
use log::trace;
use std::ffi::OsString;
use std::process::exit;

use crate::config::load_style;
use crate::modes::common::Mode;
use crate::setup::{make_window_tree, tell_gtk_about_options};

mod config;
mod input;
mod modes;
mod setup;

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

lazy_static::lazy_static! {
    static ref ARGS: Args = Args::parse();
    static ref CONFIG: Config = config::load_config(ARGS.config.as_ref());
}

fn main() {
    pretty_env_logger::init();
    trace!("starting tehda");

    let app = Application::builder()
        .application_id("page.mikufan.tehda")
        .build();

    // gtk my behated
    tell_gtk_about_options(&app);

    let modes: Vec<&str> = ARGS.modes.split(',').collect();

    if modes.is_empty() {
        error!("No modes specified. Try `drun`!");
        exit(1);
    } else if modes.contains(&"dump_config") {
        trace!("dumping config and exiting");
        println!("{}", CONFIG.serialize());
        exit(0);
    }

    app.connect_activate(move |app| {
        trace!("building window");
        // TODO: this works, but gtk starts spewing `CRITICAL`s into stdout
        let win = ApplicationWindow::builder()
            .application(app)
            .default_width(CONFIG.width)
            .default_height(CONFIG.height)
            .title("tehda")
            .window_position(gtk::WindowPosition::None)
            .gravity(gdk::Gravity::Center)
            .decorated(false)
            .resizable(false)
            .has_focus(true)
            // TODO: we probably don't actually need all events
            .events(gdk::EventMask::ALL_EVENTS_MASK)
            .build();

        // set up layer shell stuff
        gtk_layer_shell::init_for_window(&win);
        gtk_layer_shell::set_layer(&win, gtk_layer_shell::Layer::Top);
        gtk_layer_shell::set_keyboard_interactivity(&win, true);

        // apply styles
        let css_provider = load_style(&ARGS.style);
        let css_context = gtk::StyleContext::new();
        css_context.add_provider(&css_provider, 1);

        if let Some(screen) = gdk::Screen::default() {
            gtk::StyleContext::add_provider_for_screen(
                &screen,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        };

        // keeping this exchange here for posterity despite it being very outdated:
        // <autumn>: if i change [this type] to a ref it gets mad about me moving the value
        //           in there and then has a whole bunch of lifetime bullshit going on
        // <ash>: THIS is the point of leaking! autumn im gonna leak just for you

        win.connect_key_press_event(|_, keypress| {
            if let Some(s) = keypress.keyval().name() {
                if s == CONFIG.keybinds.quit {
                    exit(0);
                }
            }

            gtk::Inhibit(false)
        });

        let enabled_modes = modes
            .iter()
            .filter_map(|mode| match *mode {
                "drun" => Some(Mode::Drun),
                "run" => Some(Mode::Run),
                _ => None,
            })
            .collect();

        make_window_tree(&win, enabled_modes);

        trace!("showing window");
        win.show_all();
    });

    app.run();
}
