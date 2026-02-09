use gtk::Builder;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, HeaderBar};
use gtk::{CssProvider, gdk};

pub struct Ui {
    window: ApplicationWindow,
}

impl Ui {
    pub fn new(app: &Application) -> Self {
        // Load CSS provider first
        Self::load_css();

        let builder = Builder::from_file("resources/window.ui");

        let window: ApplicationWindow =
            builder.object("main_window").expect("Failed to get window");

        let header_bar: HeaderBar = builder
            .object("headerbar")
            .expect("Failed to get header bar");

        window.set_application(Some(app));
        window.set_titlebar(Some(&header_bar));

        Self { window }
    }

    fn load_css() {
        let provider = CssProvider::new();
        provider.load_from_path("resources/style.css"); // No error handling needed

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
