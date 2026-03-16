use std::process::Command;
use std::path::Path;

fn main() {
    // Re-run build if resource files change
    println!("cargo:rerun-if-changed=resources.gresource.xml");
    println!("cargo:rerun-if-changed=resources/window.ui");
    println!("cargo:rerun-if-changed=resources/ledger_banner.ui");
    println!("cargo:rerun-if-changed=resources/style.css");
    println!("cargo:rerun-if-changed=resources/org.gtk_rs.CheckIT.gschema.xml");

    // Compile GResources
    let status = Command::new("glib-compile-resources")
        .args([
            "--target=resources/resources.gresource",
            "--generate",
            "resources.gresource.xml",
        ])
        .status()
        .expect("Failed to execute glib-compile-resources");

    if !status.success() {
        eprintln!("Resource compilation failed with exit code: {:?}", status);
        panic!("Resource compilation failed");
    }

    // Compile GSettings schema
    let schema_path = Path::new("resources");
    let status = Command::new("glib-compile-schemas")
        .arg(schema_path)
        .status()
        .expect("Failed to execute glib-compile-schemas");

    if !status.success() {
        eprintln!("Schema compilation failed with exit code: {:?}", status);
        panic!("Schema compilation failed");
    }
}
