use std::cell::Cell;

use pbt::sample_table;
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

/// 値に配列またはテーブル（再帰構造）が含まれるかを判定する。
///
/// ラウンドトリップが再帰構造を実際に検証したことを coverage gate で保証するための
/// 判定関数。
fn has_nested(value: &Value) -> bool {
    match value {
        Value::Array(items) => !items.is_empty(),
        Value::Table(table) => !table.is_empty(),
        _ => false,
    }
}

/// Table -> to_string -> from_str ラウンドトリップ。
///
/// 生成した Table を TOML に変換し、再度パースして同じ Table が得られることを検証する。
/// 再帰構造（配列・テーブル）を持つ値が一度は検証されたことを coverage gate で確認する。
#[test]
fn table_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    // coverage gate: 配列またはテーブルを含む値が一度は検証されたか
    let nested_values = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let table = sample_table(ctx);
        // 再帰構造を含む Value が存在したかを、検証前に記録する。
        // 検証の成否は assert が決めるため、値の形状そのものはここで確定している。
        let has_nested_value = table.values().any(has_nested);
        let value = Value::Table(table.clone());
        let toml_str = shiguredo_toml::to_string(&value).expect("シリアライズに成功するはず");
        let parsed = shiguredo_toml::from_str(&toml_str).expect("TOML のパースに成功するはず");
        assert!(
            tables_equal(&table, &parsed),
            "ラウンドトリップに失敗しました。\n元のテーブル: {table:?}\nシリアライズ結果:\n{toml_str}\nパース結果: {parsed:?}"
        );
        // 検証を通過した後に、再帰構造を含むケースをカウントする
        if has_nested_value {
            nested_values.set(nested_values.get() + 1);
        }
        Ok(())
    })?;
    assert!(
        nested_values.get() > 0,
        "配列またはテーブルを含む値が一度もラウンドトリップされていない\n{runner}"
    );
    Ok(())
}

/// to_string -> from_str -> to_string の冪等性。
///
/// 一度直列化して再パースした結果を再度直列化すると同一文字列が得られる。
#[test]
fn serialize_idempotent() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let table = sample_table(ctx);
        let value = Value::Table(table);
        let first = shiguredo_toml::to_string(&value).expect("シリアライズに成功するはず");
        let parsed = shiguredo_toml::from_str(&first).expect("TOML のパースに成功するはず");
        let second =
            shiguredo_toml::to_string(&Value::Table(parsed)).expect("シリアライズに成功するはず");
        assert_eq!(
            first, second,
            "直列化結果が一致しない: {first:?} <> {second:?}"
        );
        Ok(())
    })?;
    Ok(())
}
