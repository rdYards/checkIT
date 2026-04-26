mod app;
mod data;
mod p2p;
mod ui;

use adw::{
    Application, gio,
    gio::Resource,
    prelude::{ApplicationExt, ApplicationExtManual},
};
use glib;

const APP_ID: &str = "org.gtk_rs.CheckIT";
const DEFAULT_FILE_PATH: &str = "~/";

#[tokio::main]
async fn main() -> glib::ExitCode {
    // Load resources from the output directory
    let res_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/data/checkit.gresource"));
    let bytes = glib::Bytes::from(&res_bytes[..]); // Convert to &[u8]
    let res = Resource::from_data(&bytes).expect("Failed to load resources");
    gio::resources_register(&res);

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        app::build_app(app);
    });

    app.run()
}
