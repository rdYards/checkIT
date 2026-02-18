use gio::prelude::*;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
pub struct BannerInfo {
    title: String,
    lock_state: bool,
    // Add other fields as needed
}

pub struct LedgerBanner {
    button: gtk::Button,
}

impl LedgerBanner {
    pub fn new(banner_info: &BannerInfo) -> Self {
        // Create the main button
        let ledger_button = gtk::Button::new();
        ledger_button.set_property("name", "banner_box");
        ledger_button.set_hexpand(true);
        ledger_button.set_vexpand(false);

        // Drive icon
        let drive_icon = gtk::Image::from_icon_name("drive-multidisk-symbolic");
        drive_icon.set_properties(&[
            ("name", &"drive_icon"),
            ("halign", &gtk::Align::Start),
            ("hexpand", &false)
        ]);


        // Create label with title
        let label = gtk::Label::new(Some(&banner_info.title));
        label.set_properties(&[
            ("name", &"network_label"),
            ("halign", &gtk::Align::Fill),
            ("hexpand", &true)
        ]);

        // Create lock button
        // Set Lock icon
        let lock_icon = if banner_info.lock_state {
            gtk::Image::from_icon_name("padlock2-symbolic")
        } else {
            gtk::Image::from_icon_name("padlock2-open-symbolic")
        };
        lock_icon.set_property("name", "lock_icon");
        
        // Create Button
        let lock_btn = gtk::Button::new();
        lock_btn.set_child(Some(&lock_icon));
        lock_btn.set_properties(&[
            ("name", &"lock_btn"),
            ("halign", &gtk::Align::End),
            ("hexpand", &false)
        ]);
        lock_btn.set_tooltip_text(Some(if banner_info.lock_state {
            "Locked"
        } else {
            "Unlocked"
        }));

        // Create main box without the outer button
        let content_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        content_box.set_property("name", "ledger_banner");
        content_box.set_hexpand(true);
        content_box.set_homogeneous(false);

        content_box.append(&drive_icon);
        content_box.append(&label);
        content_box.append(&lock_btn);

        // Set the content box as the button's child
        ledger_button.set_child(Some(&content_box));

        Self {
            button: ledger_button,
        }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.button.upcast_ref()
    }

    // Method to connect "click" action to button
    pub fn connect_clicked<F: Fn() + 'static>(&self, f: F) {
        self.button.connect_clicked(move |_| f());
    }
}

pub fn create_ledger_banners(container: &gtk::Box) {
    // Load the test JSON file
    let file_path = "ledgers/TST1.json";
    let mut file =
        std::fs::File::open(file_path).expect(&format!("Failed to open file at {}", file_path));
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect(&format!("Failed to read file at {}", file_path));

    if let Ok(banner_info) = serde_json::from_str::<BannerInfo>(&contents) {
        for _ in 1..=5 {
            let banner = LedgerBanner::new(&banner_info);
            container.append(banner.widget());
        }
    }
}
