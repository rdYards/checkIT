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

pub struct LedgerBannerList {
    list_box: gtk::ListBox,
}

impl LedgerBannerList {
    pub fn new() -> Self {
        let list_box = gtk::ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::None);
        list_box.set_hexpand(true);
        list_box.set_vexpand(true);

        // Directly load the test JSON file
        let file_path = "ledgers/TST1.json";
        let mut file =
            std::fs::File::open(file_path).expect(&format!("Failed to open file at {}", file_path));
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect(&format!("Failed to read file at {}", file_path));

        if let Ok(banner_info) = serde_json::from_str::<BannerInfo>(&contents) {
            for _ in 1..=5 {
                let banner = LedgerBanner::new(&banner_info);
                list_box.append(banner.widget());
            }
        }

        Self { list_box }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.list_box.upcast_ref()
    }
}

pub struct LedgerBanner {
    box_content: gtk::Box,
}

impl LedgerBanner {
    pub fn new(banner_info: &BannerInfo) -> Self {
        // Drive icon
        let drive_icon = gtk::Image::from_icon_name("drive-multidisk-symbolic");

        // Create label with title
        let label = gtk::Label::new(Some(&banner_info.title));
        label.set_xalign(0.0);
        label.set_hexpand(true);

        // Create lock button
        let lock_btn = gtk::Button::new();
        let lock_icon = if banner_info.lock_state {
            gtk::Image::from_resource("/org/gtk_rs/CheckIT/icons/padlock2-symbolic.svg")
        } else {
            gtk::Image::from_resource("/org/gtk_rs/CheckIT/icons/padlock2-open-symbolic.svg")
        };
        lock_btn.set_child(Some(&lock_icon));
        lock_btn.set_tooltip_text(Some(if banner_info.lock_state {
            "Locked"
        } else {
            "Unlocked"
        }));

        // Create main box without the outer button
        let box_content = gtk::Box::new(gtk::Orientation::Horizontal, 12);

        box_content.append(&drive_icon);
        box_content.append(&label);
        box_content.append(&lock_btn);

        Self { box_content }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.box_content.upcast_ref()
    }
}
