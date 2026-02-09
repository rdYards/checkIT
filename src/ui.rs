use adw::{Application, ApplicationWindow};
use gtk::prelude::*;
use gtk::{gdk};

pub struct Ui {
    window: ApplicationWindow,
}

impl Ui {
    pub fn new(app: &Application) -> Self {
        // Load CSS provider first
        Self::load_css();

        let builder = gtk::Builder::from_resource("/org/gtk_rs/CheckIT/resources/window.ui");

        let window: ApplicationWindow =
            builder.object("main_window").expect("Failed to get window");

        window.set_application(Some(app));

        Self { window }
    }

    fn load_css() {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/org/gtk_rs/CheckIT/resources/style.css");

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
