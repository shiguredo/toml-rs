use std::io::{self, Read};

use nojson::{DisplayJson, JsonFormatter};
use shiguredo_toml::{Datetime, TomlVersion, Value};

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(1);
    }

    let version = match std::env::var("TOML_VERSION").as_deref() {
        Ok("1.0") | Err(_) => TomlVersion::V1_0,
        Ok("1.1") => TomlVersion::V1_1,
        Ok(v) => {
            eprintln!("unsupported TOML_VERSION: {v}");
            std::process::exit(1);
        }
    };

    let parsed = match shiguredo_toml::from_str_with_version(&input, version) {
        Ok(v) => v,
        Err(_) => std::process::exit(1),
    };

    // ルートテーブルは tagged JSON のオブジェクトとしてそのまま出力する
    let root = Value::Table(parsed);
    let output = nojson::json(|f| f.value(TaggedJson(&root)));
    println!("{output}");
}

/// TOML の値を toml-test の tagged JSON 形式で出力するラッパー。
struct TaggedJson<'a>(&'a Value);

impl DisplayJson for TaggedJson<'_> {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> std::fmt::Result {
        match self.0 {
            Value::String(s) => tagged(f, "string", s),
            Value::Integer(n) => tagged(f, "integer", &n.to_string()),
            Value::Float(v) => {
                let text = if v.is_nan() {
                    "nan".to_owned()
                } else if v.is_infinite() {
                    if v.is_sign_negative() {
                        "-inf".to_owned()
                    } else {
                        "inf".to_owned()
                    }
                } else {
                    v.to_string()
                };
                tagged(f, "float", &text)
            }
            Value::Boolean(b) => tagged(f, "bool", if *b { "true" } else { "false" }),
            Value::Datetime(dt) => {
                let (kind, text) = tagged_datetime(dt);
                tagged(f, kind, &text)
            }
            Value::Array(values) => f.array(|f| {
                for value in values {
                    f.element(TaggedJson(value))?;
                }
                Ok(())
            }),
            Value::Table(table) => f.object(|f| {
                f.members(
                    table
                        .iter()
                        .map(|(key, value)| (key.as_str(), TaggedJson(value))),
                )
            }),
        }
    }
}

/// スカラー値を `{"type": kind, "value": text}` 形式で出力する。
fn tagged(f: &mut JsonFormatter<'_, '_>, kind: &str, text: &str) -> std::fmt::Result {
    f.object(|f| {
        f.member("type", kind)?;
        f.member("value", text)
    })
}

/// 日時の種類と TOML 表記を返す。
fn tagged_datetime(dt: &Datetime) -> (&'static str, String) {
    let kind = match (&dt.date, &dt.time, &dt.offset) {
        (Some(_), Some(_), Some(_)) => "datetime",
        (Some(_), Some(_), None) => "datetime-local",
        (Some(_), None, None) => "date-local",
        (None, Some(_), None) => "time-local",
        // ここには到達しないが、未知の組み合わせでも datetime として出力する
        _ => "datetime",
    };
    (kind, dt.to_string())
}
