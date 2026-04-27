use glib_build_tools::compile_resources;
use std::{fs, path::{Path, PathBuf}, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=data/build/org.rdyards.CheckIT.svg");
    println!("cargo:rerun-if-changed=data/resources.gresource.xml");
    println!("cargo:rerun-if-changed=data/resources/ui/window.ui");
    println!("cargo:rerun-if-changed=data/resources/ui/placeholder.ui");
    println!("cargo:rerun-if-changed=data/resources/style.css");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let data_dir = Path::new(&out_dir).join("data");
    let build_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/build");
    let svg_path = build_dir.join("org.rdyards.CheckIT.svg");

    // Ensure directories exist
    std::fs::create_dir_all(&data_dir).expect("Failed to create output directory");
    std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");

    // Check SVG exists
    if !svg_path.exists() {
        panic!("SVG file not found at {:?}", svg_path);
    }

    // Compile GResources
    compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        data_dir.join("checkit.gresource").to_str().unwrap(),
    );

    // macOS icon generation
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        let icns_dest = build_dir.join("org.rdyards.CheckIT.icns");
        if let Err(e) = generate_macos_icon(&svg_path, &icns_dest) {
            println!("cargo:warning=Failed to generate macOS icon: {}", e);
        }
    }
}

/// Generates an .icns file from an .svg source using rsvg-convert and iconutil.
fn generate_macos_icon(svg_path: &PathBuf, icns_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // iconutil expects an .iconset directory and produces an .icns file.
    let iconset_dir = "data/build/org.rdyards.CheckIT.iconset";

    if Path::new(iconset_dir).exists() {
        fs::remove_dir_all(iconset_dir)?;
    }
    fs::create_dir_all(iconset_dir)?;

    // Standard icon sizes for macOS
    let sizes = [
        (16, "16x16"),
        (32, "32x32"),
        (64, "64x64"),
        (128, "128x128"),
        (256, "256x256"),
        (512, "512x512"),
    ];

    for (size, label) in sizes {
        let output_png = format!("{}/icon_{}.png", iconset_dir, label);

        // Using rsvg-convert to convert SVG to PNG.
        let status = Command::new("rsvg-convert")
            .arg("-w")
            .arg(size.to_string())
            .arg("-h")
            .arg(size.to_string())
            .arg(svg_path)
            .arg("-o")
            .arg(&output_png)
            .status()?;

        if !status.success() {
            return Err(format!("rsvg-convert failed for size {}x{}", size, size).into());
        }
    }

    // Using iconutil create the .icns file from the iconset directory.
    let status = Command::new("iconutil")
        .arg("-c")
        .arg("icns")
        .arg(iconset_dir)
        .status()?;

    if !status.success() {
        return Err("iconutil failed to create icns file".into());
    }

    // Move the generated .icns file to the expected location
    let generated_icns = Path::new(iconset_dir).with_extension("icns");
    if generated_icns.exists() {
        fs::rename(generated_icns, icns_path)?;
    }

    Ok(())
}
