First, install `cargo-deb` via Cargo:

```bash
cargo install cargo-deb
```

Ensure you're using a recent Rust version (1.76+ recommended). If you encounter build issues, update Rust:

```bash
rustup update
```

## Build the .deb Package

Run the following command in your project’s root directory:

```bash
cargo deb
```

This will compile a .deb file to `target/debian/`

*To build and install locally in one step:*

```bash
cargo deb --install
```