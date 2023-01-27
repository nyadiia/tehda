use std::ffi::OsString;
use std::process::exit;
use log::{info, trace, warn};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use argh::FromArgs;

mod config;

#[derive(FromArgs)]
/// Wayland launcher / menu program.
struct Args {
    /// path to config
    #[argh(option, short = 'c')]
    config: Option<OsString>,

    /// print the config (usually the default one) and exit
    #[argh(switch)]
    dump_config: bool
}

fn main() {
    pretty_env_logger::init();
    trace!("starting tehda");
    let args: Args = argh::from_env();
    let config = config::load_config(args.config);

    if args.dump_config {
        trace!("dumping config and exiting");
        println!("{}", serde_yaml::to_string(&config).unwrap()); 
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
            .default_width(320)
            .default_height(200)
            .title("tehda")
            .window_position(gtk::WindowPosition::None)
            .gravity(gdk::Gravity::Center)
            .decorated(false)
            .resizable(false)
            .has_focus(true)

            .build();
        
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
