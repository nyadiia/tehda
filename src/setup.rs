use gdk::glib::{Char, OptionArg, OptionFlags};
use gtk::prelude::*;

use crate::{input::handle_input, modes::common::Mode};

/// Set up the window.
pub fn make_window_tree(win: &gtk::ApplicationWindow, enabled_modes: Vec<Mode>) {
    // it would be kinda cool to do this in like the gtk xml thing but i have
    // no clue how it works so

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

    let scrolled_window = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
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

    // handle input
    input.connect_changed(move |i| {
        handle_input(i, &flow_box, &enabled_modes);
    });
}

/// GTK hates it when you don't do options through their system. This sucks.
pub fn tell_gtk_about_options(app: &gtk::Application) {
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
}
