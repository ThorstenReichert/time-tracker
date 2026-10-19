//! Tauri's build step. On Windows, Tauri needs Common Controls v6, which only an app
//! manifest provides. tauri-build embeds it into the binary alone, so `cargo test` binaries
//! of this crate fail with STATUS_ENTRYPOINT_NOT_FOUND. Instead, the same manifest is linked
//! into every linked target here (binary and tests); see plan.md Deviations.

fn main() {
    let windows = tauri_build::WindowsAttributes::new_without_app_manifest();
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("failed to run tauri-build");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
    }
}
