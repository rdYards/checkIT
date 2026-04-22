use glib_build_tools::compile_resources;
use std::{fs, path::Path, process::Command};

fn main() {
    // Re-run build if resource files change
    println!("cargo:rerun-if-changed=data/build/org.gtk-rs.CheckIT.svg");
    println!("cargo:rerun-if-changed=data/resources.gresource.xml");
    println!("cargo:rerun-if-changed=data/resources/ui/window.ui");
    println!("cargo:rerun-if-changed=data/resources/ui/placeholder.ui");
    println!("cargo:rerun-if-changed=data/resources/style.css");
    println!("cargo:rerun-if-changed=data/org.gtk_rs.CheckIT.gschema.xml");

    // Get the output directory
    let out_dir = std::env::var("OUT_MS_DIR").unwrap_or_else(|_| "target/debug".to_string());
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| "target/debug".to_string());
    let data_dir = Path::new(&out_dir).join("data");

    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&data_dir).expect("Failed to create output directory");

    // Compile GResources with full path
    compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        data_dir.join("checkit.gresource").to_str().unwrap(),
    );

    // macOS Icon Generation
    let target_os = std::env::var("TARGET_OS").unwrap_or_default();
    let svg_source = "data/build/org.gtk-rs.CheckIT.svg";
    let icns_dest = "data/build/org.gtk-rs.CheckIT.icns";

    if Path::new(svg_source).exists() {
        if target_os == "macos" {
            if let Err(e) = generate_macos_icon(svg_source, icns_dest) {
                println!("cargo:warning=Failed to generate macOS icon: {}", e);
            }
        } else {
            // Fallback for non-macOS builds (e.g. Linux) to satisfy bundlers
            // that expect the .icns file to exist in the metadata.
            if !Path::new(icns_dest).exists() {
                let _ = fs::copy(svg_source, icns_dest);
            }
        }
    }

    /// Generates an .icns file from an .svg source using rsvg-convert and iconutil.
    fn generate_macos_icon(
        svg_path: &str,
        _icns_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // iconutil expects an .iconset directory and produces an .icns file.
        // If we name our directory '...CheckIT.iconset', iconutil will create '...CheckIT.icns'.
        let iconset_dir = "data/build/org.gtk-rs.CheckIT.iconset";

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

            // Using rsvg-convert (from librsvg) to convert SVG to PNG.
            // This tool is widely available on both Linux and macOS.
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

        // Clean up the temporary iconset directory
        fs::remove_dir_all(iconset_dir)?;

        Ok(())
    }
}
