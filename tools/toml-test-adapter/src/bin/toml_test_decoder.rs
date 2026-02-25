use std::io::{self, Read};

use serde_json::{Map, Value as JsonValue};
use shiguredo_toml::{Datetime, Value};

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(1);
    }

    let parsed = match shiguredo_toml::from_str(&input) {
        Ok(v) => v,
        Err(_) => std::process::exit(1),
    };

    let mut root = Map::new();
    for (key, value) in parsed {
        root.insert(key, to_tagged_json(&value));
    }

    let output = JsonValue::Object(root);
    match serde_json::to_string(&output) {
        Ok(s) => println!("{s}"),
        Err(_) => std::process::exit(1),
    }
}

fn tagged(kind: &str, value: String) -> JsonValue {
    let mut map = Map::new();
    map.insert("type".to_owned(), JsonValue::String(kind.to_owned()));
    map.insert("value".to_owned(), JsonValue::String(value));
    JsonValue::Object(map)
}

fn tagged_datetime(dt: &Datetime) -> JsonValue {
    let (kind, value) = match (&dt.date, &dt.time, &dt.offset) {
        (Some(_), Some(_), Some(_)) => ("datetime", dt.to_string()),
        (Some(_), Some(_), None) => ("datetime-local", dt.to_string()),
        (Some(_), None, None) => ("date-local", dt.to_string()),
        (None, Some(_), None) => ("time-local", dt.to_string()),
        _ => ("datetime", dt.to_string()),
    };
    tagged(kind, value)
}

fn to_tagged_json(value: &Value) -> JsonValue {
    match value {
        Value::String(s) => tagged("string", s.clone()),
        Value::Integer(n) => tagged("integer", n.to_string()),
        Value::Float(f) => {
            if f.is_nan() {
                tagged("float", "nan".to_owned())
            } else if f.is_infinite() {
                if f.is_sign_negative() {
                    tagged("float", "-inf".to_owned())
                } else {
                    tagged("float", "inf".to_owned())
                }
            } else {
                tagged("float", f.to_string())
            }
        }
        Value::Boolean(b) => tagged("bool", b.to_string()),
        Value::Datetime(dt) => tagged_datetime(dt),
        Value::Array(values) => {
            let mut arr = Vec::with_capacity(values.len());
            for item in values {
                arr.push(to_tagged_json(item));
            }
            JsonValue::Array(arr)
        }
        Value::Table(table) => {
            let mut map = Map::new();
            for (key, inner) in table {
                map.insert(key.clone(), to_tagged_json(inner));
            }
            JsonValue::Object(map)
        }
    }
}
