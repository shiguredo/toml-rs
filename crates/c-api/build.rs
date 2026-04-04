use std::env;
use std::path::Path;

fn main() {
    let crate_dir = env!("CARGO_MANIFEST_DIR");

    // ルートの Cargo.toml からライブラリバージョンを取得する
    let root_toml = Path::new(crate_dir).join("../../Cargo.toml");
    let root_toml_content =
        std::fs::read_to_string(&root_toml).expect("failed to read root Cargo.toml");
    let version = root_toml_content
        .lines()
        .find(|line| line.starts_with("version"))
        .and_then(|line| line.split('"').nth(1))
        .expect("failed to find version in root Cargo.toml");
    println!("cargo:rustc-env=SHIGUREDO_TOML_VERSION={version}");

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_cpp_compat(true)
        .with_include_version(true)
        .with_include_guard("SHIGUREDO_TOML_H")
        .with_no_includes()
        .with_sys_include("stdbool.h")
        .with_sys_include("stdint.h")
        .with_sys_include("stddef.h")
        .generate()
        .expect("failed to generate C bindings")
        .write_to_file("include/shiguredo_toml.h");
}
