use argh::FromArgs;
use config::Config;
use gdk::glib::GString;
use gdk::keys::constants::Escape;
use gdk::EventKey;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use log::{info, trace, warn};
use std::ffi::OsString;
use std::process::exit;

mod config;

#[derive(FromArgs)]
/// Wayland launcher / menu program.
struct Args {
    /// path to config
    #[argh(option, short = 'c')]
    config: Option<OsString>,

    /// print the config (usually the default one) and exit
    #[argh(switch)]
    dump_config: bool,
}

fn keypress_handler_with_config(
    // <autumn>: if i change [this type] to a ref it gets mad about me moving the value
    //           in there and then has a whole bunch of lifetime bullshit going on
    // <ash>: THIS is the point of leaking! autumn im gonna leak just for you
    config: &Config,
) -> impl Fn(&ApplicationWindow, &EventKey) -> Inhibit + '_ {
    move |_window: &ApplicationWindow, keypress: &EventKey| -> Inhibit {
        if let Some(key_name) = keypress.keyval().name().map(|s| s) {
            match key_name {
                s if &s == &config.keybinds.quit => exit(0),
                _ => {}
            }
        }
        gtk::Inhibit(false)
    }
}

fn main() {
    pretty_env_logger::init();
    trace!("starting tehda");
    let args: Args = argh::from_env();
    let config = Box::leak(Box::new(config::load_config(args.config)));

    if args.dump_config {
        trace!("dumping config and exiting");
        println!("{}", serde_yaml::to_string(config).unwrap());
        exit(0);
    }

    let app = Application::builder()
        .application_id("page.mikufan.tehda")
        .build();

    app.connect_activate(|app| {
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
            .focus_on_map(true)
            // TODO: we probably don't actually need all events
            .events(gdk::EventMask::ALL_EVENTS_MASK)
            .build();
        // <autumn>: i just fixed it by cloning it
        //         : config won't change during runtime so it's fine
        win.connect_key_press_event(keypress_handler_with_config(config));

        gtk_layer_shell::init_for_window(&win);

        gtk_layer_shell::set_layer(&win, gtk_layer_shell::Layer::Overlay);

        let label = gtk::Label::new(Some(""));

        label.set_markup("<span font_desc=\"20.0\">haii</span>");
        win.add(&label);
        win.set_border_width(12);

        trace!("showing window");
        win.show_all();
    });

    app.run();
}
