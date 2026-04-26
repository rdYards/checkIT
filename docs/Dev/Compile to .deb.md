To create a Debian package (.deb) for CheckIT, follow these steps:
## Prerequisites

* `cargo-deb` installed
* Rust 1.76 or later
* Required build dependencies (libgtk-4-dev, libadwaita-1-dev, etc.)
## Build Process

1. Install the cargo-deb plugin if you haven't already:
```bash
cargo install cargo-deb
```

2. Build the Debian package:
```bash
cargo deb
```

3. The resulting .deb file will be in `target/debian/`
## Installation

To install the package after building:
```bash
sudo dpkg -i target/debian/checkit_*.deb
```
## Troubleshooting

If you encounter dependency issues:
```bash
sudo apt --fix-broken install
```
