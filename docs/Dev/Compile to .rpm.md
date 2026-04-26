To create an RPM package (.rpm) for CheckIT, follow these steps
## Prerequisites

* `cargo-generate-rpm` installed
* Rust 1.76 or later
* Required build dependencies (gtk4-devel, libadwaita-devel, etc.)
## Build Process

1. Install the cargo-generate-rpm plugin if you haven't already:
```bash
cargo install cargo-generate-rpm
```

2. Build the RPM package:
```bash
cargo generate-rpm
```

3. The resulting .rpm file will be in `target/generate-rpm/`
## Installation

To install the package after building:
```bash
sudo dnf install target/generate-rpm/checkit-*.rpm
```
## Troubleshooting

If you encounter dependency issues, you may need to install additional packages:
```bash
sudo dnf builddep checkit
