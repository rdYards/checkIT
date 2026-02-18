use crate::components::ledger_banner::create_ledger_banners;
use adw::{Application, ApplicationWindow};
use gtk::gdk;
use gtk::prelude::*;

pub struct Ui {
    window: ApplicationWindow,
}

impl Ui {
    pub fn new(app: &Application) -> Self {
        // Load CSS provider first
        Self::load_css();

        let builder = gtk::Builder::from_resource("/org/gtk_rs/CheckIT/window.ui");

        let window: ApplicationWindow =
            builder.object("main_window").expect("Failed to get window");

        // Get the container from main window
        let container: gtk::Box = builder
            .object("ledger_banner_container")
            .expect("Failed to get container");
        
        // Create the ledger banner list
        create_ledger_banners(&container);

        window.set_application(Some(app));

        Self { window }
    }

    fn load_css() {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/org/gtk_rs/CheckIT/style.css");

        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
