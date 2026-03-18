use adw::{Application, ApplicationWindow, gio};
use gio::Settings;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Builder, Widget, gdk};

use crate::APP_ID;

pub struct App {
    pub window: ApplicationWindow,
    settings: Settings,
}

impl App {
    pub fn new(app: &Application) -> Self {
        // Load CSS provider first
        Self::load_css();

        let builder = Builder::new();

        // Load window.ui
        builder
            .add_from_resource("/org/gtk_rs/CheckIT/window.ui")
            .expect("Failed to load window.ui");
        // Load placeholder.ui
        builder
            .add_from_resource("/org/gtk_rs/CheckIT/placeholder.ui")
            .expect("Failed to load placeholder.ui");

        // Main window from window.ui
        let window: ApplicationWindow = builder
            .object("main_window")
            .expect("Failed to get main_window");

        // The empty placeholder container from window.ui
        let placeholder_box: GtkBox = builder
            .object("placeholder_box")
            .expect("Failed to get placeholder_box");

        // The actual placeholder content from placeholder.ui
        let placeholder_root: Widget = builder
            .object("placeholder_root")
            .expect("Failed to get placeholder_root");

        // Insert the placeholder UI into the placeholder box
        placeholder_box.append(&placeholder_root);

        window.set_application(Some(app));

        // Initialize settings
        let settings = Settings::new(APP_ID);

        Self { window, settings }
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

    fn settings(&self) -> &Settings {
        &self.settings
    }

    // TODO! Need to implement at a later data
    // pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
    //     // Get the size of the window
    //     let size = self.window.default_size();

    //     // Set the window state in `settings`
    //     self.settings().set_int("window-width", size.0)?;
    //     self.settings().set_int("window-height", size.1)?;
    //     self.settings()
    //         .set_boolean("is-maximized", self.window.is_maximized())?;

    //     Ok(())
    // }

    pub fn load_window_size(&self) {
        // Get the window state from `settings`
        let width = self.settings().int("window-width");
        let height = self.settings().int("window-height");
        let is_maximized = self.settings().boolean("is-maximized");

        // Set the size of the window
        self.window.set_default_size(width, height);

        // If the window was maximized when it was closed, maximize it again
        if is_maximized {
            self.window.maximize();
        }
    }
}
