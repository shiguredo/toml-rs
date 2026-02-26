#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    // 任意入力に対して TOML v1.1 パーサーがパニックしないことを検証する。
    let _ = shiguredo_toml::from_str_with_version(data, shiguredo_toml::TomlVersion::V1_1);
});
