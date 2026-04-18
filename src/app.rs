use adw::{
    AboutDialog, AlertDialog, Application, ApplicationWindow, EntryRow, PasswordEntryRow,
    PreferencesGroup, ResponseAppearance, ViewStack,
    gdk::Display,
    gio,
    gio::{ActionEntry, Settings},
    glib::clone,
    prelude::*,
};
use gtk::{Builder, CssProvider, FileDialog, IconTheme, License, ListBox, Widget};
use std::{rc::Rc, sync::Arc};

use crate::{
    APP_ID,
    data::{data_model::DataModel, ledger_db::LedgerDatabase},
    ui::page::PageManager,
};

/// Builds and runs the main application.
pub fn build_app(app: &Application) {
    // Import icon themes to use
    let display = Display::default().expect("Couldn't get default display");
    let icon_theme = IconTheme::for_display(&display);
    icon_theme.add_resource_path("/org/gtk_rs/CheckIT/");

    // Load CSS provider
    load_css();

    // Set up Shortcuts for Actions
    setup_shortcuts(app);

    let builder = Builder::new();

    // Load window.ui for main page
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

    // Create DataModel, used for reactive UI state
    let data_model = DataModel::new(db.clone());

    // Create PageManager, used to manage Ledger Pages
    let page_manager = PageManager::new(
        view_stack.clone(),
        button_container,
        data_model.clone(),
        db.clone(),
    );

    // Create placeholder.ui for app startup
    view_stack.add_titled(&placeholder_root, Some("placeholder"), "No Ledgers");
    view_stack.set_visible_child_name("placeholder");

    window.set_application(Some(app));
    window.set_icon_name(Some("org.gtk-rs.CheckIT"));

    let settings = Settings::new(APP_ID);
    load_window_size(&window, &settings);

    setup_actions(&window, db, page_manager);

    window.present();
}

/// Sets up global application shortcuts.
fn setup_shortcuts(app: &Application) {
    app.set_accels_for_action("win.close", &["<Ctrl>w"]);
    app.set_accels_for_action("win.show-about", &["<Ctrl>a"]);

    // Ledger Shortcuts
    app.set_accels_for_action("win.new-ledger", &["<Ctrl>n"]);
    app.set_accels_for_action("win.load-ledger", &["<Ctrl>o"]);
    app.set_accels_for_action("win.save-ledger", &["<Ctrl>s"]);
    app.set_accels_for_action("win.save-as-ledger", &["<Ctrl><Shift>s"]);
    app.set_accels_for_action("win.share-ledger", &["<Ctrl>n"]);
    app.set_accels_for_action("win.remove-ledger", &["<Ctrl>Delete"]);
    app.set_accels_for_action("win.clone-ledger", &["<Ctrl><Shift>c"]);

    // Navigation shortcuts
    app.set_accels_for_action("win.next-ledger", &["<Ctrl>Tab"]);
    app.set_accels_for_action("win.prev-ledger", &["<Ctrl><Shift>Tab"]);
}

/// Sets up application actions (e.g., "new-ledger", "load-ledger").
fn setup_actions(window: &ApplicationWindow, db: Arc<LedgerDatabase>, manager: Rc<PageManager>) {
    window.add_action_entries([
        // Action to show About dialog
        ActionEntry::builder("show-about")
            .activate(clone!(
                #[weak]
                window,
                move |_, _, _| {
                    let about = AboutDialog::builder()
                        .application_name("CheckIT")
                        .application_icon("org.gtk-rs.CheckIT")
                        .version(env!("CARGO_PKG_VERSION"))
                        .license_type(License::Gpl20)
                        .website("https://github.com/gtk-rs/checkit")
                        .issue_url("https://github.com/rdYards/checkIT/issues")
                        .developer_name("Alexander Eastman")
                        .build();

                    about.present(Some(&window));
                }
            ))
            .build(),
        // Action to create new Ledger
        ActionEntry::builder("new-ledger")
            .activate(clone!(
                #[weak]
                window,
                #[strong]
                db,
                move |_, _, _| {
                    let dialog =
                        AlertDialog::new(Some("Create New Ledger"), Some("Please enter details"));
                    dialog.add_response("cancel", "Cancel");
                    dialog.add_response("create", "Create");
                    dialog.set_response_appearance("create", ResponseAppearance::Suggested);
                    dialog.set_response_appearance("cancel", ResponseAppearance::Destructive);

                    let title_entry = EntryRow::new();
                    title_entry.set_title("Title");
                    let desc_entry = EntryRow::new();
                    desc_entry.set_title("Description");
                    let pass_entry = PasswordEntryRow::new();
                    pass_entry.set_title("Password");

                    let content = PreferencesGroup::new();
                    content.add(&title_entry);
                    content.add(&desc_entry);
                    content.add(&pass_entry);
                    dialog.set_extra_child(Some(&content));

                    let db_clone = db.clone();
                    let window_clone_close = window.clone();

                    dialog.choose(
                        Some(&window_clone_close),
                        None::<&gio::Cancellable>,
                        move |response| {
                            if response == "create" {
                                let title = title_entry.text().to_string();
                                let desc = desc_entry.text().to_string();
                                let pass = pass_entry.text().to_string();
                                if title.is_empty() || pass.is_empty() {
                                    popup_alert(&window, "Error", "Fields cannot be empty");
                                    return;
                                }
                                if let Err(e) = db_clone.create_ledger(title, desc, pass) {
                                    popup_alert(&window, "Error", &e);
                                }
                            }
                        },
                    );
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
                    let file_dialog = FileDialog::new();
                    file_dialog.set_title("Select Ledger");

                    let window_clone_close = window.clone();
                    let db_clone = db.clone();

                    file_dialog.open(
                        Some(&window_clone_close),
                        None::<&gio::Cancellable>,
                        move |result| {
                            if let Ok(file) = result {
                                let path = file
                                    .path()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();

                                let dialog = AlertDialog::new(
                                    Some("Enter Password"),
                                    Some("Enter password to import ledger"),
                                );
                                let pass_entry = PasswordEntryRow::new();
                                dialog.set_extra_child(Some(&pass_entry));
                                dialog.add_response("cancel", "Cancel");
                                dialog.add_response("import", "Import");
                                dialog
                                    .set_response_appearance("import", ResponseAppearance::Default);
                                dialog.set_response_appearance(
                                    "cancel",
                                    ResponseAppearance::Destructive,
                                );

                                let window_clone_close = window.clone();

                                dialog.choose(
                                    Some(&window_clone_close),
                                    None::<&gio::Cancellable>,
                                    move |response| {
                                        if response == "import" {
                                            if let Err(e) = db_clone.import_ledger(
                                                path.clone(),
                                                pass_entry.text().to_string(),
                                            ) {
                                                popup_alert(&window, "Import Error", &e);
                                            }
                                        }
                                    },
                                );
                            }
                        },
                    );
                }
            ))
            .build(),
        // Action to save current ledger
        ActionEntry::builder("save-ledger")
            .activate(clone!(
                #[strong]
                manager,
                #[strong]
                db,
                #[weak]
                window,
                move |_, _, _| {
                    let key = manager.state.borrow().current_ledger_key.clone();
                    if let Some(k) = key {
                        on_save_ledger(window, db.clone(), &k);
                    }
                }
            ))
            .build(),
        //  Action to save current ledger as...
        ActionEntry::builder("save-as-ledger")
            .activate(clone!(
                #[strong]
                manager,
                #[strong]
                db,
                #[weak]
                window,
                move |_, _, _| {
                    let key = manager.state.borrow().current_ledger_key.clone();
                    if let Some(k) = key {
                        on_save_as_ledger(&window, db.clone(), &k);
                    }
                }
            ))
            .build(),
        // Action to share current ledger
        ActionEntry::builder("share-ledger")
            .activate(clone!(
                #[weak]
                window,
                move |_, _, _| {
                    popup_alert(&window, "Share", "Coming soon in a future update!");
                }
            ))
            .build(),
        // Action to remove current ledger
        ActionEntry::builder("remove-ledger")
            .activate(clone!(
                #[weak]
                window,
                #[strong]
                manager,
                #[strong]
                db,
                move |_, _, _| {
                    let key = manager.state.borrow().current_ledger_key.clone();
                    if let Some(k) = key {
                        let db_clone = db.clone();
                        glib::MainContext::default().spawn_local(async move {
                            let is_imported = {
                                if let Some(ledger) = db_clone.get_ledger_data(&k) {
                                    !ledger.data.meta.root_path.to_string_lossy().contains("~/")
                                        || ledger.data.meta.root_path.to_string_lossy().len() > 2
                                } else {
                                    false
                                }
                            };

                            let dialog = AlertDialog::new(
                                Some("Remove Ledger"),
                                Some("Do you want to save changes before removing?"),
                            );
                            dialog.add_response("cancel", "Cancel");
                            dialog.add_response("no", "Remove Without Saving");
                            dialog.add_response("yes", "Save First");
                            dialog.set_response_appearance("no", ResponseAppearance::Destructive);
                            dialog.set_response_appearance("yes", ResponseAppearance::Suggested);
                            dialog.set_response_appearance("cancel", ResponseAppearance::Default);

                            let db_clone = db_clone.clone();
                            let window_clone_close = window.clone();
                            dialog.choose(
                                Some(&window_clone_close),
                                None::<&gio::Cancellable>,
                                move |response| {
                                    if response != "cancel" {
                                        if response == "yes" {
                                            if is_imported {
                                                on_save_ledger(window, db_clone.clone(), &k);
                                            } else {
                                                on_save_as_ledger(&window, db_clone.clone(), &k);
                                            }
                                            let _ = db_clone.remove_ledger(&k);
                                        } else if response == "no" {
                                            let _ = db_clone.remove_ledger(&k);
                                        }
                                    }
                                },
                            );
                        });
                    }
                }
            ))
            .build(),
        // Action to clone current ledger
        ActionEntry::builder("clone-ledger")
            .activate(clone!(
                #[weak]
                window,
                #[strong]
                db,
                #[strong]
                manager,
                move |_, _, _| {
                    let key = manager.state.borrow().current_ledger_key.clone();
                    if let Some(old_key) = key {
                        let dialog = AlertDialog::new(
                            Some("Clone Ledger"),
                            Some("Enter details for the new ledger instance"),
                        );
                        dialog.add_response("cancel", "Cancel");
                        dialog.add_response("clone", "Clone");
                        dialog.set_response_appearance("clone", ResponseAppearance::Suggested);
                        dialog.set_response_appearance("cancel", ResponseAppearance::Destructive);

                        let title_entry = EntryRow::new();
                        title_entry.set_title("New Title");

                        let pass_entry = PasswordEntryRow::new();
                        pass_entry.set_title("New Password");

                        let content = PreferencesGroup::new();
                        content.add(&title_entry);
                        content.add(&pass_entry);
                        dialog.set_extra_child(Some(&content));

                        let db_clone = db.clone();
                        let window_clone_close = window.clone();

                        dialog.choose(
                            Some(&window_clone_close),
                            None::<&gio::Cancellable>,
                            move |response| {
                                if response == "clone" {
                                    let new_title = title_entry.text().to_string();
                                    let new_pass = pass_entry.text().to_string();

                                    if new_title.is_empty() || new_pass.is_empty() {
                                        popup_alert(
                                            &window,
                                            "Error",
                                            "Title and Password cannot be empty",
                                        );
                                        return;
                                    }

                                    if let Err(e) =
                                        db_clone.clone_ledger(&old_key, new_title, new_pass)
                                    {
                                        popup_alert(&window, "Clone Error", &e.to_string());
                                    }
                                }
                            },
                        );
                    }
                }
            ))
            .build(),
        // Action to cycle to the next ledger
        ActionEntry::builder("next-ledger")
            .activate(clone!(
                #[strong]
                manager,
                move |_, _, _| {
                    manager.cycle_ledger(true);
                }
            ))
            .build(),
        // Action to cycle to the previous ledger
        ActionEntry::builder("prev-ledger")
            .activate(clone!(
                #[strong]
                manager,
                move |_, _, _| {
                    manager.cycle_ledger(false);
                }
            ))
            .build(),
        ActionEntry::builder("show-keybinds")
            .activate(clone!(
                #[weak]
                window,
                move |_, _, _| {
                    let keybinds_text = "\
                            <Ctrl>w : Close Window\n\
                            <Ctrl>a : About\n\
                            <Ctrl>n : New Ledger\n\
                            <Ctrl>o : Load Ledger\n\
                            <Ctrl>s : Save Ledger\n\
                            <Ctrl><Shift>s : Save As...\n\
                            <Ctrl><Shift>c : Clone Ledger\n\
                            <Ctrl>Delete : Remove Ledger\n\
                            <Ctrl>Tab : Next Ledger\n\
                            <Ctrl><Shift>Tab : Previous Ledger";

                    let dialog = AlertDialog::new(Some("Keybindings"), Some(keybinds_text));
                    dialog.add_response("ok", "OK");
                    dialog.set_default_response(Some("ok"));
                    dialog.present(Some(&window));
                }
            ))
            .build(),
    ]);
}

fn on_save_ledger(window: ApplicationWindow, db: Arc<LedgerDatabase>, key: &str) {
    let key = key.to_string();
    let window_clone = window.clone();
    glib::MainContext::default().spawn_local(async move {
        // 1. Verify if it was imported (has a root path that isn't default)
        let is_imported = {
            let _info = db.get_ledger_info(&key);
            // In your current Ledger implementation, we check if root_path was set.
            // Since get_ledger_info only returns BannerInfo, we check the full data.
            if let Some(ledger) = db.get_ledger_data(&key) {
                // Assuming "~/" is the default
                !ledger.data.meta.root_path.to_string_lossy().contains("~/")
                    || ledger.data.meta.root_path.to_string_lossy().len() > 2
            } else {
                false
            }
        };

        // Is file has yet to be saved to file then run "Save As"
        if !is_imported {
            on_save_as_ledger(&window_clone, db, &key);
            return;
        }

        let dialog = AlertDialog::new(
            Some("Save Ledger"),
            Some("Enter password to encrypt and save the file"),
        );
        let pass_entry = PasswordEntryRow::new();
        pass_entry.set_title("Password");

        let group = PreferencesGroup::new();
        group.add(&pass_entry);
        dialog.set_extra_child(Some(&group));
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("save", "Save");
        dialog.set_response_appearance("save", ResponseAppearance::Suggested);

        let db_inner = db.clone();
        let key_inner = key.clone();
        let win_inner = window_clone.clone();
        let win_inner_close = win_inner.clone();
        dialog.choose(
            Some(&win_inner),
            None::<&gio::Cancellable>,
            move |response| {
                if response == "save" {
                    let password = pass_entry.text().to_string();
                    if password.is_empty() {
                        return;
                    }

                    if let Err(e) = db_inner.save_ledger_to_disk(&key_inner, password) {
                        popup_alert(&win_inner_close, "Save Error", &e);
                    } else {
                        popup_alert(&win_inner_close, "Success", "Ledger saved successfully");
                    }
                }
            },
        );
    });
}

fn on_save_as_ledger(window: &ApplicationWindow, db: Arc<LedgerDatabase>, key: &str) {
    let key = key.to_string();
    let window_clone = window.clone();
    let file_dialog = gtk::FileDialog::new();
    file_dialog.set_title("Export Ledger");

    let window_clone_close = window_clone.clone();
    file_dialog.save(
        Some(&window_clone_close.clone()),
        None::<&gio::Cancellable>,
        move |result| {
            if let Ok(file) = result {
                let path = file
                    .path()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let dialog = AlertDialog::new(
                    Some("Encrypt Ledger"),
                    Some("Enter password for the new file"),
                );
                let pass_entry = PasswordEntryRow::new();
                pass_entry.set_title("Password");
                let group = PreferencesGroup::new();
                group.add(&pass_entry);
                dialog.set_extra_child(Some(&group));
                dialog.add_response("cancel", "Cancel");
                dialog.add_response("export", "Export");
                dialog.set_response_appearance("export", ResponseAppearance::Suggested);

                let path_final = path.clone();
                let win_inner = window_clone.clone();
                let win_inner_close = win_inner.clone();
                dialog.choose(
                    Some(&win_inner),
                    None::<&gio::Cancellable>,
                    move |response| {
                        if response == "export" {
                            let password = pass_entry.text().to_string();
                            if let Err(e) = db.save_ledger_as(&key, &path_final, password) {
                                popup_alert(&win_inner_close, "Export Error", &e);
                            } else {
                                popup_alert(
                                    &win_inner_close,
                                    "Success",
                                    "Ledger exported successfully",
                                );
                            }
                        }
                    },
                );
            }
        },
    );
}

/// Shows a popup alert dialog.
pub fn popup_alert(window: &ApplicationWindow, title: &str, msg: &str) {
    let dialog = AlertDialog::new(Some(title), if msg.is_empty() { None } else { Some(msg) });
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.present(Some(window));
}

/// Loads the window size from settings.
fn load_window_size(window: &ApplicationWindow, settings: &Settings) {
    let width = settings.int("window-width");
    let height = settings.int("window-height");
    let is_maximized = settings.boolean("is-maximized");
    window.set_default_size(width, height);
    if is_maximized {
        window.maximize();
    }
}

/// Loads the CSS provider.
fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_resource("/org/gtk_rs/CheckIT/style.css");
    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
