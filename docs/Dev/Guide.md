This guide provides instructions on how to set up the development environment and build the project.
## Prerequisites
Ensure you have the following installed on your system:

*   **Rust Toolchain**: The latest stable version of `rustc` and `cargo`.
*   **Build Dependencies (Linux)**:
    *   `pkg-config`
    *   `libgtk-3-dev` (or `gtk3` on some distributions)
    *   `glib2.0-dev`
    *   `libarchive-dev` (required for creating package bundles)
*   **Build Dependencies (macOS)**:
    *   `gtk+3` via Homebrew (`brew install gtk+3`)
*   **Cargo Plugins**:
    *   `cargo-deb`: For generating `.deb` packages.
    *   `cargo-rpm`: For generating `.rpm` packages.
    *   `cargo-bundle`: For creating application bundles (macOS `.app`).

## Building the Project

### Local Development
To run the application locally for development:

```bash
cargo run
```
### Building Distribution Packages

The project uses specific cargo plugins to generate distribution-ready packages.

#### Linux (.deb)
To generate a Debian package, use `cargo-deb`:

```bash
cargo deb
```
#### Linux (.rpm)
To generate an RPM package, use `cargo-rpm`:

```bash
cargo rpm
```
#### macOS (.app)
To generate a macOS application bundle, use `cargo-bundle`:

```bash
cargo bundle --release
```
## CI/CD Pipeline

Our CI/CD pipeline is managed via GitHub Actions (`.github/workflows/release.yml`). The pipeline automatically handles the creation of:
*   `.deb` packages for Debian/Ubuntu.
*   `.rpm` packages for Fedora/RHEL/CentOS.
*   macOS `.app` bundles.

All artifacts are automatically uploaded to the GitHub Releases page upon a successful build.