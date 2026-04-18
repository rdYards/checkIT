CheckIT utilizes **Libadwaita** to provide a modern, adaptive, and user-centric interface. The implementation focuses on high-level widgets that handle complex UI patterns like adaptive layouts and modern dialogs.

## UI Layout & Adaptive Design

### `AdwNavigationSplitView`
The backbone of the application's layout. It manages the relationship between the **Sidebar** (containing the ledger list) and the **Content Area** (the ledger details). It allows the application to transition from a dual-pane view on desktops to a single-pane view on mobile/narrow windows.

### `AdwBreakpoint`
Used within `window.ui` to define responsive behavior. Specifically, it is used to collapse the `split_view` when the window width falls below `700sp`, ensuring the UI remains usable on small screens.

### `AdwNavigationPage`
Used to define distinct navigable areas within the `SplitView`. The application uses these to separate the "Ledger Navigation" (sidebar) from the "Ledger Content" (the active ledger details).

### `AdwToolbarView`
Provides a structured way to manage the top and bottom bars of the content area. In CheckIT, this is used to house the `AdwHeaderBar` and the main scrollable content.

## Interactive Components

### `AdwStatusPage`
Implemented in `placeholder.ui`. It provides a polished "Empty State" when no ledgers are present in the database, featuring a centered icon, title, description, and call-to-action buttons.

### `AdwAlertDialog`
The primary mechanism for user interaction for critical decisions. It is used throughout `app.rs` for:
- Error reporting.
- Confirmation of destructive actions (e.g., removing a ledger).
- Input collection (e.g., entering passwords for import/export).

### `AdwActionRow` & `AdwPreferencesGroup`
Used within dialogs and the "Keybindings" view to present information in a clean, grouped, and standardized format. These components allow for easy addition of `EntryRow` and `PasswordEntryRow` for user input.
