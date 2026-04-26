# Creating a Flatpak Package

This guide provides instructions for creating a Flatpak package for CheckIT.

## Prerequisites

Before you begin, ensure you have the following installed:

* `flatpak-builder`
* Python 3.11 or later
* `pip` for Python package management

## Setup

1. First, install the required Python dependencies:
```bash
pip install tomlkit aiohttp
```

2. Generate a lockfile for the Python dependencies:
```bash
uv venv
uv add tomlkit aiohttp
uv lock -o uv.lock
```

## Building the Flatpak

1. Run the flatpak-builder command to create the bundle:
```bash
flatpak-builder --force-clean target/flatpak org.rdyards.CheckIT.json
```

## Installation

After building, you can install the Flatpak locally:

1. Install the bundle:
```bash
flatpak install target/flatpak/org.rdyards.CheckIT.*.flatpak
```

## Development Workflow

For development, you can use the following commands:

1. Clean build:
```bash
flatpak-builder --force-clean target/flatpak org.rdyards.CheckIT.json
```

2. Build in developer mode (with permissions):
```bash
flatpak-builder --user --install --force-clean target/flatpak org.rdyards.CheckIT.json
```
## Troubleshooting

If you encounter issues with the build:

1. Check that all dependencies are installed:
```bash
flatpak install flathub org.gtk.Gtk3theme.Adwaita
flatpak install flathub org.gtk.Platform.Gtk3
```

2. Verify the JSON manifest file is properly configured

3. Check the build logs for specific errors:
```bash
cat target/flatpak/build/org.rdyards.CheckIT.log
```

The resulting `.flatpak` file can be found in the `target/flatpak/` directory and can be distributed through various channels including: