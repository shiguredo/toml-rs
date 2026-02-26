#![no_main]
use libfuzzer_sys::fuzz_target;
use shiguredo_toml::Value;

fuzz_target!(|data: &str| {
    // パース成功した場合にシリアライザがパニックしないことを検証する。
    let Ok(table) = shiguredo_toml::from_str(data) else {
        return;
    };

    let value = Value::Table(table);

    // 通常シリアライズ
    let _ = shiguredo_toml::to_string(&value);
    // 整形シリアライズ
    let _ = shiguredo_toml::to_string_pretty(&value);
    // インライン値シリアライズ
    let _ = shiguredo_toml::to_inline_string(&value);
});
