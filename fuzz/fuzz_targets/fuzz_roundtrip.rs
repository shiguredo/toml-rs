#![no_main]
use libfuzzer_sys::fuzz_target;
use shiguredo_toml::Value;

fuzz_target!(|data: &str| {
    // パース成功した場合、直列化して再パースしても同等の結果が得られることを検証する。
    if let Ok(table) = shiguredo_toml::from_str(data) {
        let value = Value::Table(table);
        if let Ok(serialized) = shiguredo_toml::to_string(&value) {
            // 再パースがパニックしないことを検証
            let _ = shiguredo_toml::from_str(&serialized);
        }
    }
});
