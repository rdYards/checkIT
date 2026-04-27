# CheckIT - Secure Ledger Management Application

CheckIT is a secure ledger management application built with GTK-4 and Libadwaita. It provides a modern, user-friendly interface for creating, loading, and managing encrypted ledgers.

## Features

- **Secure Ledger Management**: Create and manage encrypted ledgers with password protection
- **Modern UI**: Built with Libadwaita for a clean, adaptive interface
- **Multiple Ledgers**: Manage multiple ledgers simultaneously with a navigation sidebar
- **Cross-platform**: Works on Linux, Windows, and macOS

## Installation

Still in development

## Development

### Prerequisites
- Rust (1.65 or later)
- GTK-4 and Libadwaita development libraries

# Installation

CheckIT can be installed in several ways depending on your preferences and operating system.

The easiest way to install CheckIT is by downloading a pre-built package from the [GitHub Releases page](https://github.com/rdYards/checkIT/releases).

### macOS
1. Download the `CheckIT-macos.zip` file from the latest release
2. Move the CheckIT app to your Applications folder: `mv CheckIT.app /Applications/`
3. Open the app from Launchpad or Finder

Note: This is not part of the MacOS Developer Program and the app may trigger a security warning when first opened. You can bypass this by:
1. Opening System Settings
2. Going to Privacy & Security
3. Scrolling down to the "Open Anyway" button for CheckIT

### Linux (Debian/Ubuntu)
1. Download the `.deb` file for your architecture
2. Install it using dpkg: `sudo dpkg -i checkit*.deb`
3. If you get dependency errors, run: `sudo apt --fix-broken install`

### Linux (Fedora/RHEL/CentOS)
1. Download the `.rpm` file for your architecture
2. Install it using dnf: `sudo dnf install checkit*.rpm`
3. Or using yum: `sudo yum install checkit*.rpm`

### Flatpak
1. Download the `.flatpak` file
2. Install it using flatpak: `flatpak install checkit.flatpak`

## Manual Installation

If you prefer to build from source or want the latest development version, follow these steps:

### Prerequisites

* Rust (1.76 or later recommended)
* Cargo
* GTK4 development libraries
* Libadwaita development libraries

On Ubuntu/Debian:
```bash
sudo apt-get install libgtk-4-dev libadwaita-1-dev pkg-config
```

On Fedora:
```bash
sudo dnf install gtk4-devel libadwaita-devel
```

On macOS:
```bash
brew install gtk4 libadwaita
```

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/rdYards/checkIT.git
cd checkIT
```

2. Build the application:
```bash
cargo build --release
```

3. The binary will be created at `target/release/checkit`

### Running the Application

```bash
./target/release/checkit
```

### Creating a Desktop Entry (Linux)

If you want to create a desktop entry to launch the app from your application menu:

1. Create a file at `~/.local/share/applications/checkit.desktop` with the following content:
```
[Desktop Entry]
Name=CheckIT
Exec=/path/to/target/release/checkit
Icon=/path/to/icon.svg
Terminal=false
Type=Application
```

2. Make it executable:
```bash
chmod +x ~/.local/share/applications/checkit.desktop
```

## Troubleshooting

If you encounter issues with the macOS app:
- The app might not appear in Launchpad immediately. Try restarting your Mac or check in Finder under Applications
- If you get a " damaged" warning, you may need to manually bypass it through System Settings > Privacy & Security
