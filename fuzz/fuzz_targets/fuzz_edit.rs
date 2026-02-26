#![no_main]
use libfuzzer_sys::fuzz_target;
use shiguredo_toml::{Document, Value};

fuzz_target!(|data: &str| {
    // 任意の TOML 入力に対して Document の編集操作がパニックしないことを検証する。
    let Ok(mut doc) = Document::parse(data) else {
        return;
    };

    // ルートキーの挿入
    let _ = doc.set_path("__fuzz_key", Value::Integer(1));

    // ネストしたキーの挿入（中間テーブル自動作成）
    let _ = doc.set_path("__fuzz_parent.__fuzz_child", Value::String("x".into()));

    // 深いネストの挿入
    let _ = doc.set_path("__a.__b.__c.__d", Value::Boolean(true));

    // 既存キーの上書きを試みる（テーブル内の最初のキーがあれば）
    if let Some((key, _)) = doc.as_table().iter().next() {
        let key = key.clone();
        let _ = doc.set_path(&key, Value::Integer(999));
    }
});
