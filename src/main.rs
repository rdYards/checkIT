mod ui;

use crate::ui::Ui;
use gtk::Application;
use gtk::glib;
use gtk::prelude::*;

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("org.gtk_rs.TestIdea")
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let window = Ui::new(app);
    window.present();
}
