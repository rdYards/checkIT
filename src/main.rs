mod ledger;
mod ui;

use crate::ui::Ui;
use gio::prelude::*;
use gtk::glib;
use gtk::prelude::*;

struct Manifest {}

fn main() -> glib::ExitCode {
    // Pull resources.gresource for
    let res =
        gio::Resource::load("resources/resources.gresource").expect("Failed to load resources");
    gio::resources_register(&res);

    let app = adw::Application::builder()
        .application_id("org.gtk_rs.CheckIT")
        .build();

    // Get settings
    let settings = gio::Settings::new("org.gtk_rs.CheckIT");

    // Get or set the data directory
    let data_dir = settings.string("data-directory");
    if data_dir.is_empty() {
        let mut dir = glib::user_data_dir();
        dir.push("checkit");
        settings
            .set_string("data-directory", &dir.to_str().unwrap())
            .unwrap();
    }

    // To get the directory back:
    let data_dir = settings.string("data-directory");
    println!("Data will be stored in: {}", data_dir);

    app.connect_startup(setup_shortcuts);
    app.connect_activate(build_ui);
    app.run()
}

fn setup_shortcuts(app: &adw::Application) {
    app.set_accels_for_action("win.filter('All')", &["<Ctrl>a"]);
    app.set_accels_for_action("win.filter('Open')", &["<Ctrl>o"]);
    app.set_accels_for_action("win.filter('Done')", &["<Ctrl>d"]);
}

fn build_ui(app: &adw::Application) {
    // Import icon themes to use
    let display = gtk::gdk::Display::default().expect("Couldn't get default display");
    let icon_theme = gtk::IconTheme::for_display(&display);
    icon_theme.add_resource_path("/org/gtk_rs/CheckIT/icons");

    let window = Ui::new(app);

    // Add actions
    let action_quit = gio::SimpleAction::new("quit", None);
    action_quit.connect_activate(|_, _| {
        std::process::exit(0);
    });
    app.add_action(&action_quit);

    window.present();
}
