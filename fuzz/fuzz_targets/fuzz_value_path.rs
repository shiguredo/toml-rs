#![no_main]
use libfuzzer_sys::fuzz_target;
use shiguredo_toml::parse_value_path;

fuzz_target!(|data: &str| {
    // 任意文字列に対して parse_value_path がパニックしないことを検証する。
    let _ = parse_value_path(data);
});
