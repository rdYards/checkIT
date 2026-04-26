To create a macOS application bundle (.app) for CheckIT, follow these steps:

## Prerequisites

* `cargo-bundle` installed
* Rust 1.76 or later
* Homebrew
* GTK4 and Libadwaita installed via Homebrew
## Build Process

1. Install Homebrew packages if you haven't already:
```bash
brew install gtk4 libadwaita
```

2. Install the cargo-bundle plugin if you haven't already:
```bash
cargo install cargo-bundle
```

3. Build the macOS bundle:
```bash
cargo bundle --release
```

4. The resulting application bundle will be in `target/release/bundle/osx/CheckIT.app`
## Installation

To install the application:
1. Copy the .app file to your Applications folder
2. You may need to bypass Gatekeeper warnings by:
   - Opening System Settings
   - Going to Privacy & Security
   - Scrolling down to the "Open Anyway" button for CheckIT
## Troubleshooting

If you encounter issues with the bundle, try:
```bash
codesign --deep -s - target/release/bundle/osx/CheckIT.app
