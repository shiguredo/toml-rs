use std::io::{self, Read};

use serde_json::{Map, Value as JsonValue};
use shiguredo_toml::{Datetime, Table, Value};

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(1);
    }

    let json = match serde_json::from_str::<JsonValue>(&input) {
        Ok(v) => v,
        Err(_) => std::process::exit(1),
    };

    let root = match json {
        JsonValue::Object(map) => map,
        _ => std::process::exit(1),
    };

    let mut table = Table::new();
    for (key, value) in root {
        let parsed = match from_tagged_json(&value) {
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

fn from_tagged_json(value: &JsonValue) -> Result<Value, String> {
    match value {
        JsonValue::Array(arr) => {
            let mut values = Vec::with_capacity(arr.len());
            for item in arr {
                values.push(from_tagged_json(item)?);
            }
            Ok(Value::Array(values))
        }
        JsonValue::Object(map) => parse_object(map),
        _ => Err("unsupported JSON value".to_owned()),
    }
}

fn parse_object(map: &Map<String, JsonValue>) -> Result<Value, String> {
    if map.len() == 2 && map.contains_key("type") && map.contains_key("value") {
        let kind = map
            .get("type")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| "invalid type field".to_owned())?;
        let raw = map
            .get("value")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| "invalid value field".to_owned())?;
        return parse_scalar(kind, raw);
    }

    let mut table = Table::new();
    for (key, value) in map {
        table.insert(key.clone(), from_tagged_json(value)?);
    }
    Ok(Value::Table(table))
}

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

fn parse_float(raw: &str) -> Result<f64, String> {
    let lower = raw.to_ascii_lowercase();
    match lower.as_str() {
        "inf" | "+inf" => Ok(f64::INFINITY),
        "-inf" => Ok(f64::NEG_INFINITY),
        "nan" | "+nan" | "-nan" => Ok(f64::NAN),
        _ => raw
            .parse::<f64>()
            .map_err(|_| "invalid float".to_owned()),
    }
}
