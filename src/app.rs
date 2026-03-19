use adw::{Application, ApplicationWindow, gio, glib, glib::clone};
use gio::{ActionEntry, Settings};
use gtk::prelude::*;
use gtk::{Box as GtkBox, Builder, IconTheme, Widget, gdk, gdk::Display};

use crate::{APP_ID, actions};

pub fn build_app(app: &Application) {
    // Import icon themes to use
    let display = Display::default().expect("Couldn't get default display");
    let icon_theme = IconTheme::for_display(&display);
    icon_theme.add_resource_path("/org/gtk_rs/CheckIT/icons");

    // Load CSS provider
    load_css();

    // Set up Shortcuts for Actions
    setup_shortcuts(app);

    let builder = Builder::new();

    // Load window.ui
    builder
        .add_from_resource("/org/gtk_rs/CheckIT/window.ui")
        .expect("Failed to load window.ui");

    // Load .ui for components
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

    // Set up actions
    setup_actions(&window);

    // Load window size
    load_window_size(&window, &settings);

    window.present();
}

fn setup_actions(window: &ApplicationWindow) {
    // Action to create new Ledger
    let action_new_ledger = ActionEntry::builder("new-ledger")
        .activate(clone!(
            #[weak]
            window,
            move |_, _, _| {
                actions::new_ledger(window.clone());
            }
        ))
        .build();

    // Action to load Ledger
    let action_load_ledger = ActionEntry::builder("load-ledger")
        .activate(clone!(
            #[weak]
            window,
            move |_, _, _| {
                actions::load_ledger(window.clone());
            }
        ))
        .build();

    // Add all actions to ApplicationWindow
    window.add_action_entries([action_new_ledger, action_load_ledger]);
}

fn setup_shortcuts(app: &adw::Application) {
    app.set_accels_for_action("win.close", &["<Ctrl>W"]);
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

fn load_window_size(window: &ApplicationWindow, settings: &Settings) {
    // Get the window state from `settings`
    let width = settings.int("window-width");
    let height = settings.int("window-height");
    let is_maximized = settings.boolean("is-maximized");

    // Set the size of the window
    window.set_default_size(width, height);

    // If the window was maximized when it was closed, maximize it again
    if is_maximized {
        window.maximize();
    }
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
