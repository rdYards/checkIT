mod app;
mod data;
mod p2p;
mod ui;

use adw::{
    Application, ColorScheme, StyleManager, gio,
    gio::Resource,
    prelude::{ApplicationExt, ApplicationExtManual},
};
use glib;

const APP_ID: &str = "org.rdyards.CheckIT";
const DEFAULT_FILE_PATH: &str = "~/";

#[tokio::main]
async fn main() -> glib::ExitCode {
    // Initialize GTK before creating the application
    gtk::init().expect("Failed to initialize GTK");

    // Load resources from the output directory
    let res_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/data/checkit.gresource"));
    let bytes = glib::Bytes::from(&res_bytes[..]); // Convert to &[u8]
    let res = Resource::from_data(&bytes).expect("Failed to load resources");
    gio::resources_register(&res);

    // Create the application
    let app = Application::builder().application_id(APP_ID).build();

    // Set default color scheme
    let style_manager = StyleManager::default();
    style_manager.set_color_scheme(ColorScheme::PreferLight);

    app.connect_activate(|app| {
        app::build_app(app);
    });

    app.run()
}
