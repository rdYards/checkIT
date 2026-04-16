use adw::{
    ActionRow, AlertDialog, ComboRow, EntryRow, HeaderBar, NavigationPage, PreferencesGroup,
    ResponseAppearance, ToolbarView, ViewStack, gio,
    gio::Cancellable,
    glib,
    prelude::{
        ActionRowExt, AlertDialogExt, AlertDialogExtManual, ComboRowExt, PreferencesGroupExt,
        PreferencesRowExt,
    },
};
use gtk::{
    Align, Box as GTKBox, Button, Entry, GestureClick, Image, Label, ListBox, MenuButton,
    Orientation, Popover, SelectionMode, StringList, Widget, prelude::*,
};
use sl::types::LedgerEntry;
use std::{cell::RefCell, collections, collections::HashMap, rc::Rc, sync::Arc};

use crate::{
    data::data_model::{DataModel, UiLedger},
    data::ledger_db::LedgerBannerInfo,
    data::ledger_db::LedgerDatabase,
};

pub struct PageManagerState {
    pub pages: HashMap<String, NavigationPage>,
    pub list_containers: HashMap<String, GTKBox>,
    pub button_map: HashMap<String, GTKBox>,
    pub description_labels: HashMap<String, Label>,
    pub selected_entry: Option<String>,
    pub search_query: String,
    pub current_ledger_key: Option<String>,
}

#[derive(Clone)]
pub struct PageManager {
    view_stack: ViewStack,
    button_container: ListBox,
    data_model: DataModel,
    db: Arc<LedgerDatabase>,
    // Separate state to allow safe mutation in async blocks
    pub state: Rc<RefCell<PageManagerState>>,
}

impl PageManager {
    pub fn new(
        view_stack: ViewStack,
        button_container: ListBox,
        data_model: DataModel,
        db: Arc<LedgerDatabase>,
    ) -> Rc<Self> {
        let manager = Rc::new(Self {
            view_stack,
            button_container,
            data_model,
            db,
            state: Rc::new(RefCell::new(PageManagerState {
                pages: HashMap::new(),
                list_containers: HashMap::new(),
                button_map: HashMap::new(),
                description_labels: HashMap::new(),
                selected_entry: None,
                search_query: String::new(),
                current_ledger_key: None,
            })),
        });

        manager.clone().subscribe_to_data_model();
        manager
    }

    /// Subscribes to DataModel changes and updates the UI.
    fn subscribe_to_data_model(self: Rc<Self>) {
        let mut ledgers_rx = self.data_model.ledgers.clone();

        glib::MainContext::default().spawn_local(glib::clone!(
            #[strong(rename_to = manager)]
            self,
            #[strong(rename_to = vstack)]
            self.view_stack,
            #[strong(rename_to = btn_ctn)]
            self.button_container,
            async move {
                println!("[DEBUG PageManager] Started subscription to DataModel");

                while let Ok(()) = ledgers_rx.changed().await {
                    println!("[DEBUG PageManager] DataModel changed, getting updated ledgers");

                    let ledgers = ledgers_rx.borrow().clone();
                    println!("[DEBUG PageManager] Received {} ledgers", ledgers.len());

                    // Collect current state
                    let current_keys: collections::HashSet<_>;
                    let list_containers: HashMap<String, GTKBox>;
                    {
                        let state = manager.state.borrow(); // Immutable borrow
                        current_keys = state.pages.keys().cloned().collect();
                        list_containers = state.list_containers.clone();
                    }

                    let new_keys: collections::HashSet<_> =
                        ledgers.iter().map(|l| l.banner.key.clone()).collect();

                    let to_add: Vec<UiLedger> = new_keys
                        .difference(&current_keys)
                        .map(|key| {
                            ledgers
                                .iter()
                                .find(|l| &l.banner.key == key)
                                .unwrap()
                                .clone()
                        })
                        .collect();

                    let to_remove: Vec<String> =
                        current_keys.difference(&new_keys).cloned().collect();

                    let to_update: Vec<&UiLedger> = ledgers
                        .iter()
                        .filter(|l| current_keys.contains(&l.banner.key))
                        .collect();

                    // Remove old ledgers
                    for key in to_remove {
                        let page;
                        let button;
                        let container;
                        {
                            let mut state = manager.state.borrow_mut();
                            page = state.pages.remove(&key);
                            button = state.button_map.remove(&key);
                            container = state.list_containers.remove(&key);
                        }

                        if let Some(page) = page {
                            vstack.remove(&page);
                        }
                        if let Some(button) = button {
                            button.unparent();
                        }
                        if let Some(container) = container {
                            container.unparent();
                        }
                    }

                    // Add new ledgers
                    for ledger in to_add {
                        let key = ledger.banner.key.clone();
                        let (page, container) =
                            PageManager::create_page_components(&manager, &ledger)
                                .expect("Failed to create page components");

                        let button =
                            PageManager::create_navigation_button(&ledger.banner, &key, &manager)
                                .unwrap();

                        let key_for_ui = key.clone();

                        // Insert into state
                        {
                            let mut state = manager.state.borrow_mut();
                            state.pages.insert(key.clone(), page.clone());
                            state.button_map.insert(key.clone(), button.clone());
                            state.list_containers.insert(key, container.clone());
                        }

                        vstack.add_named(&page, Some(&key_for_ui));
                        btn_ctn.append(&button);
                    }

                    // Update existing ledgers
                    for ledger in to_update {
                        let key = ledger.banner.key.clone();
                        // Update description text
                        {
                            let state = manager.state.borrow();
                            if let Some(label) = state.description_labels.get(&key) {
                                let new_desc = ledger
                                    .data
                                    .as_ref()
                                    .map(|d| d.data.meta.description.clone())
                                    .unwrap_or_default();
                                label.set_text(&new_desc);
                            }
                        }

                        // Update LedgerEntry list
                        if let Some(container) = list_containers.get(&ledger.banner.key) {
                            if let Some(ref data) = ledger.data {
                                let search_query = {
                                    let state = manager.state.borrow();
                                    state.search_query.clone()
                                };

                                // Filter entries based on current search query
                                let filtered_entries = if search_query.is_empty() {
                                    data.data.ledger.to_vec()
                                } else {
                                    data.data
                                        .ledger
                                        .iter()
                                        .filter(|entry| {
                                            entry
                                                .id
                                                .to_lowercase()
                                                .contains(&search_query.to_lowercase())
                                                || entry
                                                    .genre
                                                    .to_lowercase()
                                                    .contains(&search_query.to_lowercase())
                                                || entry
                                                    .data
                                                    .to_lowercase()
                                                    .contains(&search_query.to_lowercase())
                                        })
                                        .cloned()
                                        .collect()
                                };

                                if let Err(e) = PageManager::build_entry_list(
                                    &manager,
                                    &filtered_entries,
                                    container,
                                ) {
                                    println!("[DEBUG] Error rebuilding entry list: {}", e);
                                }
                            }
                        }
                    }

                    manager.reveal_placeholder();
                }
            }
        ));
    }

    // Method to update ledger's description
    fn on_update_description(&self, key: &str, new_description: &str) {
        let db = self.db.clone();
        let key = key.to_string();
        let new_description = new_description.to_string();
        let view_stack = self.view_stack.clone();

        glib::MainContext::default().spawn_local(async move {
            match db.update_ledger_description(&key, new_description) {
                Ok(_) => {} // DataModel will handle the UI update via subscription
                Err(e) => {
                    PageManager::page_popup_alert(
                        &view_stack,
                        "Error",
                        &format!("Failed to update description: {}", e),
                    );
                }
            }
        });
    }

    // Add a method to handle entry creation
    fn on_add_entry(&self, key: &str, genre: &str, data: &str) {
        let db = self.db.clone();
        let key = key.to_string();
        let genre = genre.to_string();
        let data = data.to_string();
        let view_stack = self.view_stack.clone();

        glib::MainContext::default().spawn_local(async move {
            match db.add_entry_to_ledger(key, genre, data) {
                Ok(_) => {} // DataModel will handle the UI update via subscription
                Err(e) => {
                    PageManager::page_popup_alert(
                        &view_stack,
                        "Error adding entry",
                        &format!("Failed to add entry: {}", e),
                    );
                }
            }
        });
    }

    // Add a method to handle entry removal
    fn on_remove_entry(&self, key: &str, entry_id: &str) {
        let db = self.db.clone();
        let key = key.to_string();
        let entry_id = entry_id.to_string();
        let view_stack = self.view_stack.clone();

        glib::MainContext::default().spawn_local(async move {
            match db.remove_entry_from_ledger(key, entry_id) {
                Ok(_) => {} // DataModel will handle the UI update via subscription
                Err(e) => {
                    PageManager::page_popup_alert(
                        &view_stack,
                        "Error removing entry",
                        &format!("Failed to remove entry: {}", e),
                    );
                }
            }
        });
    }

    // Pages
    /// Shows the page with the given key.
    fn show_page(&self, key: &str) {
        let page = {
            let state = self.state.borrow_mut();
            state.pages.get(key).cloned()
        };

        if let Some(page) = page {
            self.view_stack.set_visible_child(&page);
            self.highlight_active_button(key);

            // Then update the current ledger key
            self.state.borrow_mut().current_ledger_key = Some(key.to_string());
        }
    }

    /// Reveals the placeholder if no ledgers are available.
    fn reveal_placeholder(&self) {
        let has_pages = {
            let state = self.state.borrow();
            !state.pages.is_empty()
        };

        if !has_pages {
            self.view_stack.set_visible_child_name("placeholder");
            return;
        }

        // Check current page validity
        let current_child = self.view_stack.visible_child();
        let is_current_page_valid = {
            let state = self.state.borrow();
            current_child.map_or(false, |child| {
                state
                    .pages
                    .values()
                    .any(|p| std::ptr::eq(p.upcast_ref::<Widget>().as_ptr(), child.as_ptr()))
            })
        };

        if is_current_page_valid {
            return;
        }

        // Get first key
        let first_key = {
            let state = self.state.borrow();
            state.pages.keys().next().cloned()
        };

        if let Some(first_key) = first_key {
            self.show_page(&first_key);
        }
    }

    // Buttons
    /// Highlights the active button.
    fn highlight_active_button(&self, key: &str) {
        let state = self.state.borrow();
        for button in state.button_map.values() {
            button.remove_css_class("active");
        }
        if let Some(button) = state.button_map.get(key) {
            button.add_css_class("active");
        }
    }

    // Create Components
    /// Creates the page components for a ledger
    fn create_page_components(
        manager: &PageManager,
        ledger: &UiLedger,
    ) -> Result<(NavigationPage, GTKBox), glib::Error> {
        let toolbar_view = ToolbarView::new();
        let header_bar = HeaderBar::new();
        header_bar.set_show_title(true);
        header_bar.set_show_back_button(false);
        header_bar.set_show_end_title_buttons(true);

        let about_button = Button::new();
        about_button.set_icon_name("help-about-symbolic");
        about_button.set_tooltip_text(Some("About"));
        about_button.set_action_name(Some("win.show-about"));

        header_bar.pack_end(&about_button);
        toolbar_view.add_top_bar(&header_bar);

        let content_box = GTKBox::new(Orientation::Vertical, 0);
        content_box.set_margin_bottom(10);
        content_box.set_margin_start(10);
        content_box.set_margin_end(10);
        toolbar_view.set_content(Some(&content_box));

        // Description
        let description_text = ledger
            .data
            .as_ref()
            .map(|d| d.data.meta.description.clone())
            .unwrap_or_default();

        let description_label = Label::new(Some(&description_text));
        description_label.set_wrap(true);
        description_label.set_margin_start(10);
        description_label.set_margin_end(10);
        description_label.set_margin_bottom(10);
        description_label.add_css_class("dim-label");

        let gesture = GestureClick::new();
        let manager_clone = manager.clone();
        let key_clone = ledger.banner.key.clone();
        gesture.connect_pressed(glib::clone!(
            #[strong]
            manager_clone,
            #[strong]
            key_clone,
            move |_, _, _, _| {
                let dialog = AlertDialog::new(
                    Some("Edit Description"),
                    Some("Enter a new description for this ledger"),
                );
                let entry_row = EntryRow::new();
                entry_row.set_title("Description");
                entry_row.set_text(&description_text);

                let content = PreferencesGroup::new();
                content.add(&entry_row);
                dialog.set_extra_child(Some(&content));

                dialog.add_response("cancel", "Cancel");
                dialog.add_response("save", "Save");
                dialog.set_response_appearance("save", ResponseAppearance::Suggested);
                dialog.set_response_appearance("cancel", ResponseAppearance::Destructive);

                let manager_inner = manager_clone.clone();
                let key_inner = key_clone.clone();
                dialog.choose(
                    Some(&manager_clone.view_stack),
                    None::<&gio::Cancellable>,
                    move |response| {
                        if response == "save" {
                            manager_inner.on_update_description(&key_inner, &entry_row.text());
                        }
                    },
                );
            }
        ));
        description_label.add_controller(gesture);
        content_box.append(&description_label);

        // Register the widget
        let key = ledger.banner.key.clone();
        manager
            .state
            .borrow_mut()
            .description_labels
            .insert(key, description_label.clone());

        // Action toolbar
        let action_toolbar = GTKBox::new(Orientation::Horizontal, 10);
        action_toolbar.set_halign(Align::Fill);
        action_toolbar.set_valign(Align::Center);
        action_toolbar.set_css_classes(&["action_bar", "card"]);
        content_box.append(&action_toolbar);

        // Create "Add Entry" button
        let add_button = Button::new();
        add_button.set_icon_name("list-add-symbolic");
        add_button.set_tooltip_text(Some("Add new entry"));
        add_button.add_css_class("suggested-action");
        action_toolbar.append(&add_button);

        add_button.connect_clicked(glib::clone!(
            #[strong]
            manager,
            move |_| {
                // Define genre tags (related to Gitops)
                let git_tags = vec![
                    "add", "remove", "modify", "commit", "push", "pull", "init", "merge", "branch",
                    "stash", "reset", "tag", "fetch", "clone", "status", "log", "diff",
                ];

                // Show a dialog to enter entry details
                let dialog = AlertDialog::new(Some("Add Entry"), Some("Enter entry details"));

                // Create a dropdown for Git tags
                let genre_combo = ComboRow::new();
                genre_combo.set_title("Genre");

                // Create a string list for the dropdown
                let list = StringList::new(&git_tags);
                genre_combo.set_model(Some(&list));

                let content = PreferencesGroup::new();
                content.add(&genre_combo);
                let data_entry = EntryRow::new();
                data_entry.set_title("Data");
                content.add(&data_entry);
                dialog.set_extra_child(Some(&content));

                dialog.add_response("cancel", "Cancel");
                dialog.add_response("add", "Add");
                dialog.set_response_appearance("add", ResponseAppearance::Suggested);
                dialog.set_response_appearance("cancel", ResponseAppearance::Destructive);

                let manager_clone = manager.clone();
                dialog.choose(
                    Some(&manager.view_stack),
                    None::<&gio::Cancellable>,
                    move |response| {
                        if response == "add" {
                            let genre = genre_combo
                                .selected_item()
                                .and_then(|item| {
                                    item.downcast_ref::<gtk::StringObject>()
                                        .map(|obj| obj.string().to_string())
                                })
                                .unwrap_or_default();
                            let data = data_entry.text().to_string();

                            // Validate all fields
                            if genre.is_empty() || data.is_empty() {
                                PageManager::page_popup_alert(
                                    &manager_clone.view_stack,
                                    "Error",
                                    "All fields must be filled",
                                );
                                return;
                            }

                            let key = manager_clone.state.borrow().current_ledger_key.clone();
                            if let Some(key) = key {
                                manager_clone.on_add_entry(&key, &genre, &data);
                            }
                        }
                    },
                );
            }
        ));

        // Create "Remove Entry" button
        let remove_button = Button::new();
        remove_button.set_icon_name("edit-delete-symbolic");
        remove_button.set_tooltip_text(Some("Remove selected entry"));
        remove_button.add_css_class("destructive-action");
        action_toolbar.append(&remove_button);

        remove_button.connect_clicked(glib::clone!(
            #[strong]
            manager,
            move |_| {
                let (ledger_key, selected_entry) = {
                    let state = manager.state.borrow();
                    (
                        state.current_ledger_key.clone(),
                        state.selected_entry.clone(),
                    )
                };

                if let Some(ledger_key) = ledger_key {
                    if let Some(entry_id) = selected_entry {
                        let dialog = AlertDialog::new(
                            Some("Remove Entry"),
                            Some(&format!(
                                "Are you sure you want to remove entry {}?",
                                entry_id
                            )),
                        );
                        let content = PreferencesGroup::new();
                        let label = Label::new(Some("This action cannot be undone."));
                        content.add(&label);
                        dialog.set_extra_child(Some(&content));

                        dialog.add_response("cancel", "Cancel");
                        dialog.add_response("remove", "Remove");
                        dialog.set_response_appearance("remove", ResponseAppearance::Destructive);
                        dialog.set_response_appearance("cancel", ResponseAppearance::Suggested);

                        let manager_clone = manager.clone();
                        let ledger_key_clone = ledger_key.clone();
                        let entry_id_clone = entry_id.clone();

                        dialog.choose(
                            Some(&manager.view_stack),
                            None::<&gio::Cancellable>,
                            move |response| {
                                if response == "remove" {
                                    manager_clone
                                        .on_remove_entry(&ledger_key_clone, &entry_id_clone);

                                    manager_clone.state.borrow_mut().selected_entry = None;
                                }
                            },
                        );
                    }
                }
            }
        ));

        // ledger_content && ledger_content_container are out of order to work with search_entry
        let ledger_content = GTKBox::new(Orientation::Vertical, 12);
        ledger_content.set_hexpand(true);
        ledger_content.set_vexpand(true);

        let ledger_content_container = GTKBox::new(Orientation::Vertical, 12);
        ledger_content_container.set_hexpand(true);
        ledger_content_container.set_vexpand(true);

        // Add search bar
        let search_entry = Entry::new();
        search_entry.set_placeholder_text(Some("Search by genre, id, or data"));
        search_entry.set_hexpand(true);
        search_entry.set_margin_start(10);
        action_toolbar.append(&search_entry);

        search_entry.connect_changed(glib::clone!(
            #[strong]
            manager,
            #[strong]
            ledger_content_container,
            move |search_entry| {
                let search_query = search_entry.text().to_string();

                let current_key = {
                    let state = manager.state.borrow();
                    state.current_ledger_key.clone()
                };

                if let Some(current_key) = current_key {
                    manager.state.borrow_mut().search_query = search_query.clone();
                    let ledger_data = manager.db.get_ledger_data(&current_key);

                    if let Some(ledger) = ledger_data {
                        let filtered_entries = if search_query.is_empty() {
                            ledger.data.ledger.to_vec()
                        } else {
                            ledger
                                .data
                                .ledger
                                .iter()
                                .filter(|entry| {
                                    entry
                                        .id
                                        .to_lowercase()
                                        .contains(&search_query.to_lowercase())
                                        || entry
                                            .genre
                                            .to_lowercase()
                                            .contains(&search_query.to_lowercase())
                                        || entry
                                            .data
                                            .to_lowercase()
                                            .contains(&search_query.to_lowercase())
                                })
                                .cloned()
                                .collect()
                        };

                        // Rebuild the entry list with filtered data
                        if let Err(e) = PageManager::build_entry_list(
                            &manager,
                            &filtered_entries,
                            &ledger_content_container,
                        ) {
                            println!("[DEBUG] Error rebuilding entry list: {}", e);
                        }
                    }
                }
            }
        ));

        // Build entry list (pure UI)
        if let Some(ref data) = ledger.data {
            PageManager::build_entry_list(&manager, &data.data.ledger, &ledger_content_container)?;
        }

        content_box.append(&ledger_content_container);
        content_box.append(&ledger_content);

        let page = NavigationPage::new(
            &toolbar_view,
            &ledger
                .data
                .as_ref()
                .map(|d| d.data.meta.title.clone())
                .unwrap_or_default(),
        );

        Ok((page, ledger_content_container))
    }

    fn build_entry_list(
        manager: &PageManager,
        entries: &[LedgerEntry],
        container: &GTKBox,
    ) -> Result<(), glib::Error> {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        if entries.is_empty() {
            let no_results_label = Label::new(Some("No matching entries found"));
            no_results_label.add_css_class("dim-label");
            no_results_label.set_margin_top(20);
            container.append(&no_results_label);
            return Ok(());
        }

        let list_box = ListBox::new();
        list_box.set_hexpand(true);
        list_box.set_vexpand(true);
        list_box.set_margin_top(6);
        list_box.set_margin_bottom(6);
        list_box.add_css_class("transparent-list");
        list_box.set_selection_mode(SelectionMode::Single);

        for entry in entries {
            let row = ActionRow::new();
            row.set_title(&entry.id);
            row.set_subtitle(&entry.timestamp);

            let content_box = GTKBox::new(Orientation::Horizontal, 12);
            content_box.set_halign(Align::Start);
            content_box.set_margin_start(6);
            content_box.set_margin_end(6);

            let genre_label = Label::new(Some(&entry.genre));
            genre_label.add_css_class("genre-pill");
            genre_label.add_css_class(&format!("genre-{}", entry.genre.to_lowercase()));

            let data_label = Label::new(Some(&entry.data));
            data_label.set_wrap(true);

            content_box.append(&genre_label);
            content_box.append(&data_label);

            row.set_child(Some(&content_box));
            list_box.append(&row);
        }

        // Connect to the selection change signal
        list_box.connect_selected_rows_changed(glib::clone!(
            #[strong]
            manager,
            move |list_box| {
                let selected_rows = list_box.selected_rows();

                if let Some(row) = selected_rows.first() {
                    if let Some(action_row) = row.downcast_ref::<ActionRow>() {
                        manager.state.borrow_mut().selected_entry =
                            Some(action_row.title().to_string());
                    }
                } else {
                    manager.state.borrow_mut().selected_entry = None;
                }
            }
        ));

        container.append(&list_box);

        Ok(())
    }

    /// Creates NavBtn linked to Page
    fn create_navigation_button(
        info: &LedgerBannerInfo,
        key: &str,
        manager: &PageManager,
    ) -> Result<GTKBox, glib::Error> {
        let container = GTKBox::new(Orientation::Horizontal, 10);
        container.set_property("name", "ledger_banner");
        container.set_hexpand(true);
        container.add_css_class("navigation-button");

        // Add the Click Gesture for navigation
        let gesture = GestureClick::new();
        let key_for_gesture = key.to_string();
        let manager_for_gesture = manager.clone();

        gesture.connect_pressed(glib::clone!(
            #[strong]
            manager_for_gesture,
            #[strong]
            key_for_gesture,
            move |_, _, _, _| {
                manager_for_gesture.show_page(&key_for_gesture);
            }
        ));
        container.add_controller(gesture);

        // Inner Content
        let drive_icon = Image::from_icon_name("drive-multidisk-symbolic");
        drive_icon.set_property("name", "drive_icon");
        drive_icon.set_halign(Align::Start);

        let label = Label::new(Some(&info.title));
        label.set_property("name", "network_label");
        label.set_halign(Align::Fill);
        label.set_hexpand(true);

        let inner_button_content = Image::from_icon_name("application-menu-symbolic");

        let settings_btn = MenuButton::new();
        settings_btn.set_child(Some(&inner_button_content));
        settings_btn.set_property("name", "settings_btn");
        settings_btn.set_halign(Align::End);
        settings_btn.set_hexpand(false);

        // Popover Logic
        let popover = Popover::new();
        let popover_content = GTKBox::new(Orientation::Vertical, 0);
        popover_content.set_margin_start(5);
        popover_content.set_margin_end(5);
        popover_content.set_margin_top(5);
        popover_content.set_margin_bottom(5);

        let actions = vec![
            ("Save", "document-save-symbolic", "win.save-ledger"),
            ("Save As", "document-save-as-symbolic", "win.save-as-ledger"),
            ("Clone", "edit-copy-symbolic", "win.clone-ledger"),
            ("Share", "folder-publicshare-symbolic", "win.share-ledger"),
            ("Remove", "edit-delete-symbolic", "win.remove-ledger"),
        ];

        for (text, icon_name, action_id) in actions {
            let btn = Button::new();
            btn.add_css_class("flat");
            btn.set_action_name(Some(action_id));

            let btn_box = GTKBox::new(Orientation::Horizontal, 10);
            let icon = Image::from_icon_name(icon_name);
            let lbl = Label::new(Some(text));
            btn_box.append(&icon);
            btn_box.append(&lbl);
            btn.set_child(Some(&btn_box));

            popover_content.append(&btn);
        }

        popover.set_child(Some(&popover_content));
        settings_btn.set_popover(Some(&popover));

        container.append(&drive_icon);
        container.append(&label);
        container.append(&settings_btn);

        Ok(container)
    }

    /// Shows a popup alert dialog.
    fn page_popup_alert(window: &ViewStack, title: &str, msg: &str) {
        let dialog = AlertDialog::new(Some(title), if msg.is_empty() { None } else { Some(msg) });
        dialog.add_response("ok", "OK");
        dialog.set_default_response(Some("ok"));

        dialog.choose(Some(window), None::<&Cancellable>, |_| {});
    }
}
