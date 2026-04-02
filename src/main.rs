mod actions;
mod app;
mod ledger;
mod ledger_db;
mod page;

use adw::{Application, gio, gio::Resource, gio::prelude::*, glib};
use std::path::Path;

const APP_ID: &str = "org.gtk_rs.CheckIT";
const DEFAULT_FILE_PATH: &str = "~/";

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
