use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=resources.gresources.xml");
    println!("cargo:rerun-if-changed=resources/window.ui");
    println!("cargo:rerun-if-changed=resources/ledger_banner.ui");
    println!("cargo:rerun-if-changed=resources/style.css");
    println!("cargo:rerun-if-changed=resources/org.gtk_rs.CheckIT.gschema.xml");

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

    // Compile the schema
    let status = Command::new("glib-compile-schemas")
        .arg("resources")
        .status()
        .expect("Failed to compile GSettings schemas");

    if !status.success() {
        panic!("Schema compilation failed: {}", status);
    }
}
