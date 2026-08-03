use pbt::table_strategy;
use proptest::prelude::*;
use shiguredo_toml::{Table, Value};

/// 浮動小数点数の近似比較（NaN は両方 NaN なら等しいとみなす）。
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => {
            if f1.is_nan() && f2.is_nan() {
                true
            } else {
                (f1 - f2).abs() < 1e-10 || f1 == f2
            }
        }
        (Value::Array(a1), Value::Array(a2)) => {
            a1.len() == a2.len()
                && a1
                    .iter()
                    .zip(a2.iter())
                    .all(|(v1, v2)| values_equal(v1, v2))
        }
        (Value::Table(t1), Value::Table(t2)) => {
            t1.len() == t2.len()
                && t1
                    .iter()
                    .zip(t2.iter())
                    .all(|((k1, v1), (k2, v2))| k1 == k2 && values_equal(v1, v2))
        }
        _ => a == b,
    }
}

/// テーブルの近似比較。
fn tables_equal(a: &Table, b: &Table) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|((k1, v1), (k2, v2))| k1 == k2 && values_equal(v1, v2))
}

proptest! {
    /// Table -> to_string -> from_str ラウンドトリップ。
    ///
    /// 生成した Table を TOML に変換し、再度パースして同じ Table が得られることを検証する。
    #[test]
    fn table_roundtrip(table in table_strategy()) {
        let value = Value::Table(table.clone());
        let toml_str = shiguredo_toml::to_string(&value).expect("serialization should succeed");
        let parsed = shiguredo_toml::from_str(&toml_str).expect("TOML should parse");
        prop_assert!(
            tables_equal(&table, &parsed),
            "Round-trip failed.\nOriginal: {table:?}\nSerialized:\n{toml_str}\nParsed: {parsed:?}"
        );
    }

    /// to_string -> from_str -> to_string の冪等性。
    ///
    /// 一度直列化して再パースした結果を再度直列化すると同一文字列が得られる。
    #[test]
    fn serialize_idempotent(table in table_strategy()) {
        let value = Value::Table(table);
        let first = shiguredo_toml::to_string(&value).expect("serialization should succeed");
        let parsed = shiguredo_toml::from_str(&first).expect("TOML should parse");
        let second = shiguredo_toml::to_string(&Value::Table(parsed)).expect("serialization should succeed");
        prop_assert_eq!(first, second);
    }
}
