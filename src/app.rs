use adw::{
    Application, ApplicationWindow, ViewStack, gdk,
    gdk::Display,
    gio::{ActionEntry, Settings},
    glib,
    glib::clone,
};
use gtk::{Builder, IconTheme, ListBox, Widget, prelude::*};
use std::sync::Arc;

use crate::{
    APP_ID, actions,
    ledger_db::{LedgerBannerInfo, LedgerDatabase, LockEvent},
    page::PageManager,
};

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

    // Load window.ui fir main page
    builder
        .add_from_resource("/org/gtk_rs/CheckIT/window.ui")
        .expect("Failed to load window.ui");

    // Main window from window.ui
    let window: ApplicationWindow = builder
        .object("main_window")
        .expect("Failed to get main_window");

    // Load .ui for components for pages
    builder
        .add_from_resource("/org/gtk_rs/CheckIT/placeholder.ui")
        .expect("Failed to load placeholder.ui");
    builder
        .add_from_resource("/org/gtk_rs/CheckIT/ledger.ui")
        .expect("Failed to load ledger.ui");

    // The actual placeholder content from placeholder.ui
    let placeholder_root: Widget = builder
        .object("placeholder_root")
        .expect("Failed to get placeholder_root");

    // Non-Page related components
    // Get the view stack
    let view_stack: ViewStack = builder
        .object("view_stack")
        .expect("Failed to get view_stack");
    let button_container: ListBox = builder
        .object("ledger_list")
        .expect("Failed to get ledger_banner_container");
    button_container.set_property("name", "banner_box");

    // Create LedgerDatabase, used for ledger management
    let db = Arc::new(LedgerDatabase::new());

    // Create PageManager, used to manage Ledger Pages
    let mut page_manager = PageManager::new(view_stack.clone(), button_container);

    // Create placeholder.ui for app startup
    view_stack.add_titled(&placeholder_root, Some("placeholder"), "No Ledgers");
    view_stack.set_visible_child_name("placeholder");

    window.set_application(Some(app));

    let settings = Settings::new(APP_ID);

    load_window_size(&window, &settings);

    setup_actions(&window, db.clone());

    // Subscribe events for db and UI
    let mut events = db.subscribe_lock_events();
    let window_clone = window.clone();
    glib::idle_add_local(move || {
        if let Ok(events_receiver) = &mut events {
            while let Ok(event) = events_receiver.try_recv() {
                match event {
                    LockEvent::LedgerAdded(ledger_key) => {
                        if let Ok(Some(ledger)) = db.request_ledger(&ledger_key, "ui", true) {
                            let info = LedgerBannerInfo {
                                key: ledger_key.clone(),
                                title: ledger.data.meta.title.clone(),
                                state: ledger.state.clone(),
                            };
                            if let Err(e) =
                                page_manager.create_ledger_page_and_banner(&ledger, &info)
                            {
                                actions::popup_alert(
                                    &window_clone,
                                    "Error Creating Ledger",
                                    &format!("Error creating ledger page and banner: {}", e),
                                );
                            }
                            page_manager.show_page(&ledger_key);
                            page_manager.highlight_active_button(&ledger_key);
                        }
                    }
                    LockEvent::LedgerRemoved(ledger_key) => {
                        page_manager.remove_page(&ledger_key);
                    }
                    _ => {}
                }
            }
        }
        glib::ControlFlow::Continue
    });

    window.present();
}

fn setup_actions(window: &ApplicationWindow, db: Arc<LedgerDatabase>) {
    // Add all actions to ApplicationWindow
    window.add_action_entries([
        // Action to create new Ledger
        ActionEntry::builder("new-ledger")
            .activate(clone!(
                #[weak]
                window,
                #[strong]
                db,
                move |_, _, _| {
                    actions::new_ledger(window, db.clone());
                }
            ))
            .build(),
        // Action to load Ledger
        ActionEntry::builder("load-ledger")
            .activate(clone!(
                #[weak]
                window,
                #[strong]
                db,
                move |_, _, _| {
                    actions::load_ledger(window, db.clone());
                }
            ))
            .build(),
    ]);
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
    // Retrieve window state from settings
    let width = settings.int("window-width");
    let height = settings.int("window-height");
    let is_maximized = settings.boolean("is-maximized");

    window.set_default_size(width, height);

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
