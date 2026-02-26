#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    // 任意入力に対するパニック安全性を検証する。
    // エラーは許容されるが、パニックは許容されない。
    let _ = shiguredo_toml::from_str(data);
});
