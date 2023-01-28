use argh::FromArgs;
use config::Config;
use gdk::EventKey;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use log::error;
use log::trace;
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
    let args: Args = argh::from_env();
    // safety: i dont care about the leaking here. we're using the config immutably
    //         for the program's duration, so ill be damned if i dont have a &'static
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
        win.connect_key_press_event(keypress_handler_with_config(config));
        win.set_border_width(12);

        gtk_layer_shell::init_for_window(&win);

        gtk_layer_shell::set_layer(&win, gtk_layer_shell::Layer::Overlay);

        // set up the application's widgets
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let input = gtk::Entry::new();
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

            // it isn't clear from the docs, so i'll describe it here: the search
            // returns a vector of ranks, each rank being a vector of filenames
            // each rank is unsorted within itself, it just means that each
            // filename in that search has the same relatedness to the query
            // as each other
            //
            // TODO: it could pay off to sort within the ranks ourselves
            gio::DesktopAppInfo::search(query.as_str())
                .into_iter()
                .map(|rank| {
                    rank.into_iter()
                        .map(|filename| gio::DesktopAppInfo::new(filename.as_str()))
                })
                .flatten()
                // TODO: holy nesting, batman
                .for_each(|result| {
                    if let Some(info) = result {
                        let flow_box_child = gtk::FlowBoxChild::new();
                        let label_text = format!("{}", info.display_name());
                        let label = gtk::Label::new(Some(label_text.as_str()));
                        flow_box_child.add(&label);
                        flow_box.add(&flow_box_child);
                        flow_box_child.show();
                        label.show();

                        flow_box_child.connect_activate(move |_| {
                            let exec_path = info.executable();
                            match subprocess::Exec::cmd(exec_path).detached().join() {
                                Ok(_) => exit(0),
                                Err(e) => {
                                    error!("error running command: {}", e);
                                    exit(1);
                                }
                            };
                        });
                    }
                });
        });

        input.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("search"));

        win.add(&container);

        trace!("showing window");
        win.show_all();
    });

    app.run();
}
