use adw::{NavigationPage, ToolbarView, ViewStack};
use gtk::{Align, Box as GTKBox, Button, Image, Label, ListBox, Orientation, prelude::*};
use std::collections::HashMap;

use crate::{ledger::Ledger, ledger_db::LedgerBannerInfo};

#[derive(Clone)]
pub struct PageManager {
    view_stack: ViewStack,
    pages: HashMap<String, NavigationPage>,
    button_container: ListBox,
    button_map: HashMap<String, Button>,
}

impl PageManager {
    pub fn new(view_stack: ViewStack, button_container: ListBox) -> Self {
        Self {
            view_stack, // Location in .ui for PageManager managing AdwNavigationPages
            pages: HashMap::new(),
            button_container, // Location .ui for PageManager managing GTKButtons
            button_map: HashMap::new(),
            // HashMaps use same keys for button and pages to link compents together
        }
    }

    /// Creates a ledger page and its corresponding banner button
    pub fn create_ledger_page_and_banner(
        &mut self,
        ledger: &Ledger,
        info: &LedgerBannerInfo,
    ) -> Result<(), glib::Error> {
        let ledger_key = info.key.clone();

        // Create page and store in pages hashmap
        let ledger_page = self.create_page_components(ledger, &ledger_key)?;
        self.pages.insert(ledger_key.clone(), ledger_page.clone());

        self.view_stack.add_named(&ledger_page, Some(&ledger_key));

        let button = self.create_navigation_button(&info, &ledger_key)?;
        self.button_map.insert(ledger_key.clone(), button.clone());
        self.button_container.append(&button);

        // Set up click handler
        let page_manager = self.clone();
        let ledger_key_clone = ledger_key.clone();

        button.connect_clicked(move |_| {
            if let Some(page) = page_manager.pages.get(&ledger_key_clone) {
                page_manager.view_stack.set_visible_child(page);
                page_manager.highlight_active_button(&ledger_key_clone);
            }
        });

        Ok(())
    }

    /// Creates the page components for a ledger
    fn create_page_components(
        &self,
        ledger: &Ledger,
        ledger_key: &str,
    ) -> Result<NavigationPage, glib::Error> {
        let toolbar_view = ToolbarView::new();

        let content_box = GTKBox::new(Orientation::Vertical, 12);
        toolbar_view.set_content(Some(&content_box));

        let ledger_content = self.create_ledger_content(ledger);
        content_box.append(&ledger_content);

        let page = NavigationPage::new(&toolbar_view, &ledger.data.meta.title);

        // Note unsafe to store custom data attached to AdwNavigationPage
        unsafe { page.set_data("ledger-key", ledger_key.to_string()) };

        Ok(page)
    }

    /// Helper function to create ledger-specific content
    fn create_ledger_content(&self, ledger: &Ledger) -> GTKBox {
        // TODO! Modify to add ledger tools for modifing ledger data
        let container = GTKBox::new(Orientation::Vertical, 12);
        container.set_hexpand(true);
        container.set_vexpand(true);

        let title = Label::new(Some(&ledger.data.meta.title));
        title.add_css_class("title-1");
        container.append(&title);

        let description = Label::new(Some(&ledger.data.meta.description));
        description.add_css_class("body");
        description.set_wrap(true);
        container.append(&description);

        container
    }

    /// Creates NavBtn linked to Page
    fn create_navigation_button(
        &self,
        info: &LedgerBannerInfo,
        ledger_key: &str,
    ) -> Result<Button, glib::Error> {
        // Main button
        let button = Button::new();
        button.set_hexpand(true);
        button.set_vexpand(false);
        button.set_visible(true);
        button.set_can_focus(true);

        // Note unsafe to store custom data attached to AdwNavigationPage
        unsafe { button.set_data("ledger-key", ledger_key.to_string()) };

        // Inner button content
        let drive_icon = Image::from_icon_name("drive-multidisk-symbolic");
        drive_icon.set_property("name", "drive_icon");
        drive_icon.set_halign(Align::Start);
        drive_icon.set_hexpand(false);

        let label = Label::new(Some(&info.title));
        label.set_property("name", "network_label");
        label.set_halign(Align::Fill);
        label.set_hexpand(true);

        let inner_button_content = Image::from_icon_name("application-menu-symbolic");
        inner_button_content.set_hexpand(true);
        inner_button_content.set_vexpand(false);
        inner_button_content.set_tooltip_text(Some("Ledger Actions"));

        let lock_btn = Button::new();
        lock_btn.set_child(Some(&inner_button_content));
        lock_btn.set_property("name", "lock_btn");
        lock_btn.set_halign(Align::End);
        lock_btn.set_hexpand(false);

        // Main Box
        let content_box = GTKBox::new(Orientation::Horizontal, 12);
        content_box.set_property("name", "ledger_banner");
        content_box.set_hexpand(true);
        content_box.set_homogeneous(false);

        content_box.append(&drive_icon);
        content_box.append(&label);
        content_box.append(&lock_btn);

        button.set_child(Some(&content_box));
        Ok(button)
    }

    /// Show the page with the given key
    pub fn show_page(&self, key: &str) {
        if self.pages.contains_key(key) {
            self.view_stack.set_visible_child_name(key);
        }
    }

    /// Highlight the active button
    // TODO! Note not working at the moment
    pub fn highlight_active_button(&self, key: &str) {
        // Remove highlight from all buttons
        for button in self.button_map.values() {
            button.remove_css_class("active");
        }

        if let Some(button) = self.button_map.get(key) {
            button.add_css_class("active");
        }
    }

    /// Remove a page and its corresponding button
    pub fn remove_page(&mut self, key: &str) {
        if let Some(page) = self.pages.remove(key) {
            self.view_stack.remove(&page);
        }

        if let Some(button) = self.button_map.remove(key) {
            // Get the parent of the button
            if let Some(parent) = button.parent() {
                // Check if the parent is a GTKBox
                if let Some(box_parent) = parent.dynamic_cast_ref::<gtk::Box>() {
                    box_parent.remove(&button);
                } else {
                    // If not a GTKBox, remove as a generic container
                    if let Some(container) = parent.dynamic_cast_ref::<gtk::Widget>() {
                        container.unparent();
                    }
                }
            }
        }

        // Reveal placeholder.ui if no ledgers available
        if self.pages.is_empty() {
            self.view_stack.set_visible_child_name("placeholder");
        }
    }

    /// Check if there are any pages
    pub fn has_pages(&self) -> bool {
        !self.pages.is_empty()
    }

    /// Get a reference to the view stack
    pub fn view_stack(&self) -> &ViewStack {
        &self.view_stack
    }

    /// Get all available ledger keys
    pub fn get_ledger_keys(&self) -> Vec<String> {
        self.pages.keys().cloned().collect()
    }
}
