mod ui;
mod components;
use crate::ui::Ui;
use gtk::glib;
use gtk::prelude::*;

fn main() -> glib::ExitCode {
    let res =
        gio::Resource::load("resources/resources.gresource").expect("Failed to load resources");
    gio::resources_register(&res);

    let app = adw::Application::builder()
        .application_id("org.gtk_rs.CheckIT")
        .build();

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
    let window = Ui::new(app);

    // Add actions
    let action_quit = gio::SimpleAction::new("quit", None);
    action_quit.connect_activate(|_, _| {
        std::process::exit(0);
    });
    app.add_action(&action_quit);

    window.present();
}
