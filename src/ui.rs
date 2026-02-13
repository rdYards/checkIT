use crate::components::ledger_banner::LedgerBannerList;
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

        // Create the ledger banner list
        let ledger_banner_list = LedgerBannerList::new();

        // Get the list box from main window and add our banner
        let list_box: gtk::ListBox = builder
            .object("ledger_banner_list")
            .expect("Failed to get list box");

        // Replace the content of the list box with our ledger banner list
        list_box.remove_all(); // Clear existing content
        list_box.append(ledger_banner_list.widget());

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
