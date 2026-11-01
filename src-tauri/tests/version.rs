//! The app's version lives in four files. The installer takes it from `tauri.conf.json`, the
//! binary from the workspace `Cargo.toml`, and the UI's About section (R25) from
//! `ui/package.json` at build time. A release bump must change all of them together.

use std::path::Path;

fn json_version(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let json: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    json["version"]
        .as_str()
        .unwrap_or_else(|| panic!("{relative} has no version"))
        .to_owned()
}

#[test]
fn versions_agree() {
    let cargo = env!("CARGO_PKG_VERSION");
    for file in [
        "tauri.conf.json",
        "../ui/package.json",
        "../ui/package-lock.json",
    ] {
        assert_eq!(
            json_version(file),
            cargo,
            "{file} disagrees with the workspace Cargo.toml version"
        );
    }
}
