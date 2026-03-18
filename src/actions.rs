use adw::{EntryRow, PasswordEntryRow, Window};
use gtk::{
    Builder, Button, FileDialog,
    gio::{Cancellable, File},
    prelude::*,
};

pub fn new_ledger() {
    // Load the UI from the resource file
    let builder = Builder::new();
    builder
        .add_from_resource("/org/gtk_rs/CheckIT/new_ledger.ui")
        .expect("Failed to load new_ledger.ui");

    // Get the window
    let window: Window = builder
        .object("new_ledger_window")
        .expect("Failed to get new_ledger_window");

    // Get the entries and button
    let title_entry: EntryRow = builder
        .object("title_entry")
        .expect("Failed to get title_entry");

    let description_entry: EntryRow = builder
        .object("description_entry")
        .expect("Failed to get description_entry");

    let password_entry: PasswordEntryRow = builder
        .object("password_entry")
        .expect("Failed to get password_entry");

    let add_button: Button = builder
        .object("add_button")
        .expect("Failed to get add_button");

    // Connect the button's "clicked" signal
    let value = window.clone();
    add_button.connect_clicked(move |_| {
        let title = title_entry.text().to_string();
        let description = description_entry.text().to_string();
        let password = password_entry.text().to_string();
        create_ledger(title, description, password);
        
        value.close();
    });

    window.present();
}

pub fn load_ledger() {
    // Create a new file dialog
    let dialog = FileDialog::new();
    dialog.set_title("Select a Ledger File");
    dialog.set_initial_folder(Some(&File::for_path("~/")));

    // Set Filter
    // let filter = gtk::FileFilter::new();
    // filter.add_pattern("*.json");
    // dialog.set_filters(Some(&ListStore::new(FileFilter::static_type())));
    // dialog.set_default_filter(Some(&filter));

    // Present the dialog to the user
    dialog.open(Some(&Window::default()), None::<&Cancellable>, |result| {
        match result {
            Ok(file) => {
                // Get the path of the selected file
                let path = file.path().unwrap_or_default();
                println!("Selected file: {}", path.display());
                // Return or use the path as needed
            }
            Err(e) => {
                eprintln!("Error opening file dialog: {}", e);
            }
        }
    });
}

pub fn create_ledger(title: String, description: String, password: String) {
    println!(
        "Title: {}, Description: {}, Password: {}",
        title, description, password
    );
}
