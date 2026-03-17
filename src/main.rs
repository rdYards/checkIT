mod app;
mod ledger;

use crate::app::App;
use adw::glib;
use gio::prelude::*;
use gtk::prelude::*;

const APP_ID: &str = "org.gtk_rs.CheckIT";

fn main() -> glib::ExitCode {
    // Load resources from the output directory
    let resource_path = std::path::Path::new(env!("OUT_DIR")).join("data/checkit.gresource");
    let res = gio::Resource::load(resource_path).expect("Failed to load resources");
    gio::resources_register(&res);

    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_startup(setup_shortcuts);
    app.connect_activate(move |app| {
        build_app(app);
    });
    app.run()
}

fn setup_shortcuts(app: &adw::Application) {
    app.set_accels_for_action("win.filter('All')", &["<Ctrl>a"]);
    app.set_accels_for_action("win.filter('Open')", &["<Ctrl>o"]);
    app.set_accels_for_action("win.filter('Done')", &["<Ctrl>d"]);
}

fn build_app(app: &adw::Application) {
    // Import icon themes to use
    let display = gtk::gdk::Display::default().expect("Couldn't get default display");
    let icon_theme = gtk::IconTheme::for_display(&display);
    icon_theme.add_resource_path("/org/gtk_rs/CheckIT/icons");

    let app = App::new(app);

    app.load_window_size();

    app.present();
}
