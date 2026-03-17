use glib_build_tools::compile_resources;
use std::path::Path;

fn main() {
    // Re-run build if resource files change
    println!("cargo:rerun-if-changed=data/resources.gresource.xml");
    println!("cargo:rerun-if-changed=data/resources/ui/window.ui");
    println!("cargo:rerun-if-changed=data/resources/style.css");
    println!("cargo:rerun-if-changed=data/org.gtk_rs.CheckIT.gschema.xml");

    // Get the output directory
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let data_dir = Path::new(&out_dir).join("data");

    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&data_dir).expect("Failed to create output directory");

    // Compile GResources with full path
    compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        data_dir.join("checkit.gresource").to_str().unwrap(),
    );
}
