#![no_main]
use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary::{self, Arbitrary};
use shiguredo_toml::{Document, Value};

/// 構造化 fuzzing 用の入力。TOML テキストとパス文字列の両方を生成する。
#[derive(Debug, Arbitrary)]
struct EditInput {
    toml: String,
    path: String,
    /// 挿入する値の種別
    value_kind: ValueKind,
}

#[derive(Debug, Arbitrary)]
enum ValueKind {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

impl ValueKind {
    fn into_value(self) -> Value {
        match self {
            ValueKind::Integer(n) => Value::Integer(n),
            ValueKind::Float(f) => Value::Float(f),
            ValueKind::Bool(b) => Value::Boolean(b),
            ValueKind::Str(s) => Value::String(s),
        }
    }
}

fuzz_target!(|input: EditInput| {
    // パース失敗は許容する
    let Ok(mut doc) = Document::parse(&input.toml) else {
        return;
    };

    // 任意パスへの set_path がパニックしないことを検証する
    let _ = doc.set_path(&input.path, input.value_kind.into_value());
});
