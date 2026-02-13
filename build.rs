use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=resources.gresources.xml");
    println!("cargo:rerun-if-changed=resources/window.ui");
    println!("cargo:rerun-if-changed=resources/ledger_banner.ui");
    println!("cargo:rerun-if-changed=resources/style.css");

    let status = Command::new("glib-compile-resources")
        .args([
            "--target=resources/resources.gresource",
            "--generate",
            "resources.gresource.xml",
        ])
        .status()
        .expect("Install glib-compile-resources");

    if !status.success() {
        panic!("Resource compilation failed: {}", status);
    }
}
