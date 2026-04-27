use adw::{
    AboutDialog, ActionRow, AlertDialog, Application, ApplicationWindow, Dialog, EntryRow,
    HeaderBar, PasswordEntryRow, PreferencesGroup, ResponseAppearance, ViewStack, gdk::Display,
    gio, gio::ActionEntry, glib::clone, prelude::*,
};
use gtk::{
    Box as GTKBox, Builder, Button, CssProvider, FileDialog, IconTheme, Image, Label, License,
    ListBox, MenuButton, Orientation, PolicyType, Popover, ScrolledWindow, SearchEntry, Widget,
};
use std::{rc::Rc, sync::Arc};
use tokio::sync::mpsc;

use crate::{
    data::{data_model::DataModel, ledger_db::LedgerDatabase},
    p2p::{
        messenger::{IncomingTransfer, P2PManager},
        share_dialog,
        share_dialog::ShareTarget,
    },
    ui::page::PageManager,
};

/// Builds and runs the main application.
pub fn build_app(app: &Application) {
    // Import icon themes to use
    let display = Display::default().expect("Couldn't get default display");
    let icon_theme = IconTheme::for_display(&display);
    icon_theme.add_resource_path("/org/rdyards/CheckIT/");

    // Load CSS provider
    load_css();

    // Set up Shortcuts for Actions
    setup_shortcuts(app);

    let builder = Builder::new();

    // Load window.ui for main page
    builder
        .add_from_resource("/org/rdyards/CheckIT/window.ui")
        .expect("Failed to load window.ui");

    // Main window from window.ui
    let window: ApplicationWindow = builder
        .object("main_window")
        .expect("Failed to get main_window");

    // Load .ui for components for pages
    builder
        .add_from_resource("/org/rdyards/CheckIT/placeholder.ui")
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

    // Initialize the Global Menu Button
    let menu_button: MenuButton = builder.object("global_menu_button").expect("...");
    setup_global_menu(&menu_button);

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
    window.set_icon_name(Some("org.rdyards.CheckIT"));

    // Setup P2P Channel
    let (p2p_tx, mut p2p_rx) = mpsc::unbounded_channel::<IncomingTransfer>();
    let p2p_manager = Arc::new(P2PManager::new(db.clone(), p2p_tx));

    // Start P2P Agent
    let p2p_clone = p2p_manager.clone();
    tokio::spawn(async move {
        p2p_clone.start().await;
    });

    let p2p_manager_for_ui = p2p_manager.clone();

    // Handles incoming requests from other clients
    // When a request is recieved will create a popup having the
    // user respond
    let window_clone = window.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Some(transfer) = p2p_rx.recv().await {
            let p2p_manager_inner = p2p_manager_for_ui.clone();
            let win = window_clone.clone();

            let dialog = AlertDialog::new(
                Some("Incoming Transfer"),
                Some(&format!(
                    "{} wants to send you a ledger. Accept?",
                    transfer.sender_name
                )),
            );
            dialog.add_response("deny", "Deny");
            dialog.add_response("accept", "Accept");
            dialog.set_response_appearance("accept", ResponseAppearance::Suggested);

            dialog.choose(Some(&win), None::<&gio::Cancellable>, move |response| {
                if response == "accept" {
                    let stream = transfer.stream;
                    let pubkey = transfer.sender_pubkey;
                    let data_type = transfer.data_type;
                    let p2p_final = p2p_manager_inner.clone();

                    tokio::spawn(async move {
                        match p2p_final
                            .handle_incoming_stream(stream, pubkey, data_type)
                            .await
                        {
                            Ok(_decrypted_bytes) => {} // Add logging in the future
                            Err(_e) => {}              // Add logging in the future
                        }
                    });
                }
            });
        }
    });

    setup_actions(&window, db, page_manager, p2p_manager);

    window.present();
}

/// Sets up global application shortcuts.
/// Ctrl for main functions (Window, Ledger, Nav)
/// Alt used for Entries and inner-Ledger
fn setup_shortcuts(app: &Application) {
    app.set_accels_for_action("win.close", &["<Ctrl>w"]);
    app.set_accels_for_action("win.show-about", &["<Ctrl>a"]);
    app.set_accels_for_action("win.show-keybinds", &["<Ctrl>k"]);

    // Ledger Shortcuts
    app.set_accels_for_action("win.new-ledger", &["<Ctrl>n"]);
    app.set_accels_for_action("win.load-ledger", &["<Ctrl>i"]);
    app.set_accels_for_action("win.save-ledger", &["<Ctrl>s"]);
    app.set_accels_for_action("win.save-as-ledger", &["<Ctrl><Shift>s"]);
    app.set_accels_for_action("win.remove-ledger", &["<Ctrl>Delete"]);
    app.set_accels_for_action("win.clone-ledger", &["<Ctrl><Shift>c"]);

    // Sharing
    app.set_accels_for_action("win.share-ledger", &["<Ctrl>e"]);
    // Alt to follow keybinds in page.rs
    app.set_accels_for_action("win.share-entry", &["<Alt>e"]);

    // Navigation shortcuts
    app.set_accels_for_action("win.next-ledger", &["<Ctrl>Tab"]);
    app.set_accels_for_action("win.prev-ledger", &["<Ctrl><Shift>Tab"]);
}

/// Sets up application actions (e.g., "new-ledger", "load-ledger").
fn setup_actions(
    window: &ApplicationWindow,
    db: Arc<LedgerDatabase>,
    manager: Rc<PageManager>,
    p2p: Arc<P2PManager>,
) {
    window.add_action_entries([
        // Action to show About dialog
        ActionEntry::builder("show-about")
            .activate(clone!(
                #[weak]
                window,
                move |_, _, _| {
                    let about = AboutDialog::builder()
                        .application_name("CheckIT")
                        .application_icon("org.rdyards.CheckIT")
                        .version(env!("CARGO_PKG_VERSION"))
                        .license_type(License::Gpl20)
                        .website("https://github.com/rdyards/checkit")
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
                #[strong]
                p2p,
                #[strong]
                manager,
                move |_, _, _| {
                    let key = manager.state.borrow().current_ledger_key.clone();

                    if let Some(k) = key {
                        share_dialog::open_share_dialog(
                            window,
                            p2p.clone(),
                            ShareTarget::FullLedger { key: k.clone() },
                        );
                    } else {
                        popup_alert(
                            &window,
                            "Error",
                            "Please open a ledger before trying to share it",
                        );
                    }
                }
            ))
            .build(),
        // Action to share a single entry
        ActionEntry::builder("share-entry")
            .activate(clone!(
                #[weak]
                window,
                #[strong]
                manager,
                #[strong]
                p2p,
                move |_, _, _| {
                    // Access the PageManager state to see what is currently selected
                    let state = manager.state.borrow();

                    if let (Some(key), Some(entry_id)) =
                        (&state.current_ledger_key, &state.selected_entry)
                    {
                        // Call the dialog function
                        share_dialog::open_share_dialog(
                            window,
                            p2p.clone(),
                            ShareTarget::SingleEntry {
                                ledger_key: key.clone(),
                                entry_id: entry_id.clone(),
                            },
                        );
                    } else {
                        // No entry selected, notify the user
                        popup_alert(
                            &window,
                            "Error",
                            "Please select an entry from the list first",
                        );
                    }
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
        // Action to show KeyBinds dialog
        ActionEntry::builder("show-keybinds")
            .activate(clone!(
                #[weak]
                window,
                move |_, _, _| {
                    let dialog = Dialog::builder().title("Keybindings").build();
                    dialog.set_width_request(600);

                    let scrolled_window = ScrolledWindow::builder()
                        .hscrollbar_policy(PolicyType::Never)
                        .min_content_height(400)
                        .min_content_width(560)
                        .propagate_natural_width(true)
                        .propagate_natural_height(true)
                        .build();

                    let main_box = GTKBox::new(Orientation::Vertical, 5);

                    let search_entry = SearchEntry::builder()
                        .placeholder_text("Search shortcuts")
                        .margin_top(10)
                        .margin_bottom(10)
                        .margin_start(10)
                        .margin_end(10)
                        .vexpand(true)
                        .build();

                    let header_bar = HeaderBar::new();
                    header_bar.set_title_widget(Some(&search_entry));
                    main_box.append(&header_bar);

                    // Data for the rows
                    let shortcut_categories = vec![
                        (
                            "General",
                            vec![
                                ("Close Window", vec!["Ctrl", "W"]),
                                ("About", vec!["Ctrl", "A"]),
                                ("Keybinds", vec!["Ctrl", "K"]),
                            ],
                        ),
                        (
                            "Ledger Management",
                            vec![
                                ("New Ledger", vec!["Ctrl", "N"]),
                                ("Load Ledger", vec!["Ctrl", "I"]),
                                ("Save Ledger", vec!["Ctrl", "S"]),
                                ("Save As...", vec!["Ctrl", "Shift", "S"]),
                                ("Share Ledger", vec!["Ctrl", "E"]),
                                ("Remove Ledger", vec!["Ctrl", "Delete"]),
                                ("Clone Ledger", vec!["Ctrl", "Shift", "C"]),
                            ],
                        ),
                        (
                            "Navigation",
                            vec![
                                ("Next Ledger", vec!["Ctrl", "Tab"]),
                                ("Previous Ledger", vec!["Ctrl", "Shift", "Tab"]),
                            ],
                        ),
                    ];

                    let mut all_rows = Vec::new();

                    for (group_title, shortcuts) in shortcut_categories {
                        let group = PreferencesGroup::builder()
                            .title(group_title)
                            .margin_start(20)
                            .margin_end(20)
                            .margin_bottom(20)
                            .build();

                        for (label_text, keys) in shortcuts {
                            let row = ActionRow::builder().title(label_text).build();

                            let key_box = GTKBox::new(Orientation::Horizontal, 4);
                            for key in keys {
                                let key_label = Label::new(Some(key));
                                key_label.add_css_class("shortcut-badge");
                                key_box.append(&key_label);
                            }

                            row.add_suffix(&key_box);
                            group.add(&row);

                            // Store the row and its label for the search callback
                            all_rows.push((row, label_text.to_string()));
                        }
                        main_box.append(&group);
                    }

                    // Implement Search Filtering across all groups
                    search_entry.connect_search_changed(clone!(move |entry| {
                        let text = entry.text().to_lowercase();
                        for (row, label) in &all_rows {
                            if label.to_lowercase().contains(&text) {
                                row.set_visible(true);
                            } else {
                                row.set_visible(false);
                            }
                        }
                    }));
                    scrolled_window.set_child(Some(&main_box));
                    dialog.set_child(Some(&scrolled_window));
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

fn setup_global_menu(menu_button: &MenuButton) {
    let popover = Popover::new();
    let popover_content = GTKBox::new(Orientation::Vertical, 0);
    popover_content.set_margin_start(5);
    popover_content.set_margin_end(5);
    popover_content.set_margin_bottom(5);
    popover_content.set_margin_top(5);

    let actions = vec![
        ("About", "preferences-system-symbolic", "win.show-about"),
        (
            "Keybindings",
            "input-keyboard-symbolic",
            "win.show-keybinds",
        ),
    ];

    for (text, icon_name, action_id) in actions {
        let btn = Button::new();
        btn.add_css_class("flat");
        btn.set_action_name(Some(action_id));

        // Fix issue where popup does not disappear when clicked
        btn.connect_clicked(glib::clone!(
            #[weak]
            popover,
            move |_| {
                popover.popdown();
            }
        ));

        let btn_box = GTKBox::new(Orientation::Horizontal, 10);
        btn_box.append(&Image::from_icon_name(icon_name));
        btn_box.append(&Label::new(Some(text)));
        btn.set_child(Some(&btn_box));

        popover_content.append(&btn);
    }

    popover.set_child(Some(&popover_content));
    menu_button.set_popover(Some(&popover));
}

/// Shows a popup alert dialog.
pub fn popup_alert(window: &ApplicationWindow, title: &str, msg: &str) {
    let dialog = AlertDialog::new(Some(title), if msg.is_empty() { None } else { Some(msg) });
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.present(Some(window));
}

/// Loads the CSS provider.
fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_resource("/org/rdyards/CheckIT/style.css");
    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
