mod app;
mod actions;
mod ledger;

use adw::{Application, glib};
use gio::Resource;
use gio::prelude::*;
use std::path::Path;

const APP_ID: &str = "org.gtk_rs.CheckIT";

fn main() -> glib::ExitCode {
    // Load resources from the output directory
    let resource_path = Path::new(env!("OUT_DIR")).join("data/checkit.gresource");
    let res = Resource::load(resource_path).expect("Failed to load resources");
    gio::resources_register(&res);

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        app::build_app(app);
    });
    app.run()
}
