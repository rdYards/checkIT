First, install the tool via Cargo:

```
cargo install cargo-generate-rpm
```

Then, build your binary:

```bash
cargo build --release
strip -s target/release/checkit
```

Then, generate the RPM:

```bash
cargo generate-rpm
```

The `.rpm` file will be created in `target/generate-rpm/`.

