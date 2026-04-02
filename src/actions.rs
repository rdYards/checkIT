use adw::{
    AlertDialog, ApplicationWindow, EntryRow, PasswordEntryRow, PreferencesGroup,
    ResponseAppearance, Window,
    gio::{Cancellable, File},
    glib::{GString, MainContext},
    prelude::{
        AdwDialogExt, AlertDialogExt, AlertDialogExtManual, PreferencesGroupExt, PreferencesRowExt,
    },
};
use gtk::{FileDialog, prelude::*};
use std::sync::Arc;

use crate::{ledger::Ledger, ledger_db::LedgerDatabase};

pub fn new_ledger(parent: ApplicationWindow, db: Arc<LedgerDatabase>) {
    let dialog = AlertDialog::new(
        Some("Create New Ledger"),
        Some("Please enter the details for your new ledger"),
    );

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("create", "Create");

    // Input fields
    let title_entry = EntryRow::new();
    title_entry.set_title("Title");
    title_entry.set_hexpand(true);

    let description_entry = EntryRow::new();
    description_entry.set_title("Description");
    description_entry.set_hexpand(true);

    let password_entry = PasswordEntryRow::new();
    password_entry.set_title("Password");
    password_entry.set_hexpand(true);

    // Add the input fields to the dialog
    let content = PreferencesGroup::new();
    content.add(&title_entry);
    content.add(&description_entry);
    content.add(&password_entry);

    dialog.set_extra_child(Some(&content));

    dialog.set_response_appearance("create", ResponseAppearance::Suggested);
    dialog.set_response_appearance("cancel", ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let parent_close_clone = parent.clone();

    // Handle the response
    dialog.choose(
        Some(&parent_close_clone),
        None::<&Cancellable>,
        move |response_id| {
            if response_id == "create" {
                let title = title_entry.text().to_string();
                let password = password_entry.text().to_string();

                // Check if required fields are empty
                if title.trim().is_empty() || password.trim().is_empty() {
                    popup_alert(
                        &parent,
                        "Error Creating Ledger",
                        "Title and password are required fields",
                    );
                    return;
                }

                let description = description_entry.text().to_string();
                let ledger = Ledger::new(&password, &title, &description);

                match db.add_ledger(title, ledger) {
                    Ok(_) => {
                        popup_alert(&parent, "Ledger Created Successfully", "");
                    }
                    Err(e) => {
                        popup_alert(&parent, "Error Creating Ledger", &format!("{}", e));
                    }
                }
            }
        },
    );
}

pub fn load_ledger(parent: ApplicationWindow, db: Arc<LedgerDatabase>) {
    let dialog = FileDialog::new();
    dialog.set_title("Select a Ledger File");
    dialog.set_initial_folder(Some(&File::for_path("~/")));

    // !TODO need to add filter for .sl files once finished
    // Set Filter
    // let filter = gtk::FileFilter::new();
    // filter.add_pattern("*.json");
    // dialog.set_filters(Some(&ListStore::new(FileFilter::static_type())));
    // dialog.set_default_filter(Some(&filter));

    // Present the dialog to the user
    dialog.open(
        Some(&Window::default()),
        None::<&Cancellable>,
        move |result| {
            match result {
                Ok(file) => {
                    // Get the path of the selected file
                    let path = file.path().unwrap_or_default();

                    // Create password dialog
                    let dialog = AlertDialog::new(
                        Some("Enter Password"),
                        Some("Enter password associated to import ledger"),
                    );

                    dialog.add_response("cancel", "Cancel");
                    dialog.add_response("enter", "Enter");

                    let password_entry = PasswordEntryRow::new();
                    password_entry.set_title("Password");
                    password_entry.set_hexpand(true);
                    dialog.set_extra_child(Some(&password_entry));

                    dialog.set_response_appearance("enter", ResponseAppearance::Default);
                    dialog.set_response_appearance("cancel", ResponseAppearance::Destructive);
                    dialog.set_default_response(Some("cancel"));
                    dialog.set_close_response("cancel");

                    let cancellable = Cancellable::new();

                    let parent_close_clone = parent.clone();

                    dialog.choose(
                        Some(&parent_close_clone),
                        Some(&cancellable),
                        move |response_id: GString| match response_id.as_str() {
                            "enter" => {
                                let password = password_entry.text().to_string();
                                let path_str = path.to_str().unwrap().to_string();

                                if let Some(ledger) = Ledger::from_file(&password, &path_str) {
                                    match db.add_ledger(path_str, ledger) {
                                        Ok(_) => {
                                            MainContext::default().invoke_local(move || {
                                                popup_alert(
                                                    &parent,
                                                    "Ledger Created Successfully",
                                                    "",
                                                );
                                            });
                                        }
                                        Err(e) => {
                                            MainContext::default().invoke_local(move || {
                                                popup_alert(
                                                    &parent,
                                                    "Error Loading Ledger",
                                                    &format!("{}", e),
                                                );
                                            });
                                        }
                                    }
                                } else {
                                    MainContext::default().invoke_local(move || {
                                        popup_alert(
                                            &parent,
                                            "Error Loading Ledger",
                                            "Invalid password or file not found",
                                        );
                                    });
                                }
                            }
                            "cancel" => popup_alert(
                                &parent,
                                "Canceled Password",
                                "Please input a password to import Ledger",
                            ),
                            _ => {
                                popup_alert(&parent, "Unexpect Response", "Something went wrong");
                            }
                        },
                    );
                }
                Err(e) => {
                    popup_alert(
                        &parent,
                        "Error Opening File",
                        &format!("Error opening file dialog: {}", e),
                    );
                }
            }
        },
    );
}

/// Pop-up for GUI notifacations
pub fn popup_alert(parent: &ApplicationWindow, alert: &str, message: &str) {
    let dialog = AlertDialog::new(
        Some(alert),
        if message == "" { None } else { Some(message) },
    );
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));

    dialog.present(Some(parent));
}
