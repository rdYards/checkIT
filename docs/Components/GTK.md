CheckIT uses **GTK4** for fundamental widgetry, event controllers, and low-level user interaction.

## Core Widgets

### Layout & Containers
- **`GtkBox`**: The primary layout engine used to stack elements vertically (in pages) or horizontally (in action bars and navigation buttons).
- **`GtkListBox`**: Used to render the list of ledgers in the sidebar and the list of entries within a ledger page. It supports selection modes and custom row styling.

### User Input & Controls
- **`GtkButton`**: Used for all primary actions, including the "New Ledger" and "Load Ledger" buttons.
- **`GtkEntry` / `GtkSearchEntry`**: Used for text input and the search functionality within the Keybindings and Ledger Entry lists.
- **`GtkPasswordEntryRow`**: A specialized input widget for secure password entry during ledger creation or decryption.
- **`GtkComboRow` / `GtkStringList`**: Used in the "Add Entry" dialog to allow users to select a "Genre" from a predefined list of tags.
- **`GtkToggleButton`**: Utilized in the header bar to trigger the file loading process.

### Menus & Overlays
- **`GtkMenuButton`**: The entry point for the "Global Menu," providing access to About and Keybindings.
- **`GtkPopover`**: Used to display context-sensitive actions (Save, Clone, Remove) when interacting with specific ledger buttons in the sidebar.

## Event Handling & Controllers

### `GtkEventControllerKey`
The engine behind the application's power-user features. It intercepts keyboard input to trigger:
- **Global Shortcuts**: (e.g., `Ctrl+N` for New Ledger).
- **Contextual Shortcuts**: (e.g., `Alt+E` to add an entry when the ledger page is focused).

### `GtkGestureClick`
Used to implement custom navigation logic. In the sidebar, a click gesture on a ledger button triggers the `PageManager` to switch the `ViewStack` to the corresponding ledger page.

### `GtkCssProvider`
Used to inject custom styling (via `style.css`) to create "pill" shapes for genres, "badges" for keybindings, and custom "active" states for navigation buttons.