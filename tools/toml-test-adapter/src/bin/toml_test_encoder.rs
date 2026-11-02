use std::io::{self, Read};

use nojson::{JsonParseError, JsonValueKind, RawJson, RawJsonValue};
use shiguredo_toml::{Datetime, Table, Value};

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(1);
    }

    let json = match RawJson::parse(&input) {
        Ok(v) => v,
        Err(_) => std::process::exit(1),
    };

    let mut table = Table::new();
    let members = match json.value().to_object() {
        Ok(v) => v,
        Err(_) => std::process::exit(1),
    };
    for (key, value) in members {
        let key = match key.to_unquoted_string_str() {
            Ok(v) => v.into_owned(),
            Err(_) => std::process::exit(1),
        };
        let parsed = match from_tagged_json(value) {
            Ok(v) => v,
            Err(_) => std::process::exit(1),
        };
        table.insert(key, parsed);
    }

    match shiguredo_toml::to_string(&Value::Table(table)) {
        Ok(out) => print!("{out}"),
        Err(_) => std::process::exit(1),
    }
}

/// toml-test の tagged JSON 形式の値を TOML の値に変換する。
fn from_tagged_json(value: RawJsonValue<'_, '_>) -> Result<Value, JsonParseError> {
    match value.kind() {
        JsonValueKind::Array => {
            let mut values = Vec::new();
            for item in value.to_array()? {
                values.push(from_tagged_json(item)?);
            }
            Ok(Value::Array(values))
        }
        JsonValueKind::Object => {
            // `type` と `value` の 2 キーだけを持つオブジェクトをスカラーとして扱う
            let is_tagged = value.to_object()?.count() == 2
                && value.to_member("type")?.optional().is_some()
                && value.to_member("value")?.optional().is_some();
            if is_tagged {
                let kind: String = value.to_member("type")?.required()?.try_into()?;
                let raw: String = value.to_member("value")?.required()?.try_into()?;
                return parse_scalar(&kind, &raw).map_err(|message| value.invalid(message));
            }

            // テーブルはキーと値の組をそのまま TOML テーブルにする
            let mut table = Table::new();
            for (key, item) in value.to_object()? {
                let key = key.to_unquoted_string_str()?.into_owned();
                table.insert(key, from_tagged_json(item)?);
            }
            Ok(Value::Table(table))
        }
        _ => Err(value.invalid("unsupported JSON value")),
    }
}

/// tagged JSON のスカラーを TOML の値に変換する。
fn parse_scalar(kind: &str, raw: &str) -> Result<Value, String> {
    match kind {
        "string" => Ok(Value::String(raw.to_owned())),
        "integer" => raw
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| "invalid integer".to_owned()),
        "float" => parse_float(raw).map(Value::Float),
        "bool" => match raw.to_ascii_lowercase().as_str() {
            "true" => Ok(Value::Boolean(true)),
            "false" => Ok(Value::Boolean(false)),
            _ => Err("invalid bool".to_owned()),
        },
        "datetime" | "datetime-local" | "date-local" | "time-local" => raw
            .parse::<Datetime>()
            .map(Value::Datetime)
            .map_err(|_| "invalid datetime".to_owned()),
        _ => Err("unknown type".to_owned()),
    }
}

/// tagged JSON の浮動小数点表記を f64 に変換する。
fn parse_float(raw: &str) -> Result<f64, String> {
    let lower = raw.to_ascii_lowercase();
    match lower.as_str() {
        "inf" | "+inf" => Ok(f64::INFINITY),
        "-inf" => Ok(f64::NEG_INFINITY),
        "nan" | "+nan" | "-nan" => Ok(f64::NAN),
        _ => raw.parse::<f64>().map_err(|_| "invalid float".to_owned()),
    }
}
