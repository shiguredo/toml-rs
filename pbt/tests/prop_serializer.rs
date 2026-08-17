use std::cell::Cell;

use pbt::{sample_bare_key, sample_datetime, sample_safe_string};

/// 文字列値のラウンドトリップ。
///
/// エスケープが必要な文字（バックスラッシュとダブルクォート）を含む文字列が
/// 一度は検証されたことを coverage gate で確認する。エスケープの往復は
/// シリアライザとパーサの整合が崩れやすい箇所であり、空振りさせない。
#[test]
fn string_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    // coverage gate: エスケープが必要な文字を含む文字列が一度は検証されたか
    let escaped_strings = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let key = sample_bare_key(ctx);
        let value = sample_safe_string(ctx);
        // エスケープが必要な文字（バックスラッシュとダブルクォート）を含むか
        let contains_escape = value.contains('\\') || value.contains('"');

        let mut table = shiguredo_toml::Table::new();
        table.insert(key.clone(), shiguredo_toml::Value::String(value.clone()));
        let serialized = shiguredo_toml::to_string(&shiguredo_toml::Value::Table(table))
            .expect("シリアライズに成功するはず");
        let parsed = shiguredo_toml::from_str(&serialized).expect("TOML のパースに成功するはず");
        assert_eq!(
            parsed
                .get(&key)
                .expect("キーは存在するはず")
                .as_str()
                .expect("値は文字列になるはず"),
            &value,
            "文字列値が一致しない: {value:?} -> {serialized:?}"
        );
        // 検証を通過した後に、エスケープ文字を含むケースをカウントする
        if contains_escape {
            escaped_strings.set(escaped_strings.get() + 1);
        }
        Ok(())
    })?;
    assert!(
        escaped_strings.get() > 0,
        "エスケープが必要な文字を含む文字列が一度も検証されていない\n{runner}"
    );
    Ok(())
}

/// 浮動小数点数のラウンドトリップ（有限値のみ）。
#[test]
fn float_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let key = sample_bare_key(ctx);
        let f = pbt::sample_f64_boundaries(ctx);
        let mut table = shiguredo_toml::Table::new();
        table.insert(key.clone(), shiguredo_toml::Value::Float(f));
        let serialized = shiguredo_toml::to_string(&shiguredo_toml::Value::Table(table))
            .expect("シリアライズに成功するはず");
        let parsed = shiguredo_toml::from_str(&serialized).expect("TOML のパースに成功するはず");
        let parsed_f = parsed
            .get(&key)
            .expect("キーは存在するはず")
            .as_float()
            .expect("値は浮動小数点数になるはず");
        assert!(
            (parsed_f - f).abs() < 1e-10 || parsed_f == f,
            "浮動小数点数の不一致: {f} -> {serialized:?} -> {parsed_f}"
        );
        Ok(())
    })?;
    Ok(())
}

/// Datetime のラウンドトリップ。
#[test]
fn datetime_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let key = sample_bare_key(ctx);
        let dt = sample_datetime(ctx);
        let mut table = shiguredo_toml::Table::new();
        table.insert(key.clone(), shiguredo_toml::Value::Datetime(dt.clone()));
        let serialized = shiguredo_toml::to_string(&shiguredo_toml::Value::Table(table))
            .expect("シリアライズに成功するはず");
        let parsed = shiguredo_toml::from_str(&serialized).expect("TOML のパースに成功するはず");
        let parsed_dt = parsed
            .get(&key)
            .expect("キーは存在するはず")
            .as_datetime()
            .expect("値は日時になるはず");
        assert_eq!(
            &parsed_dt.date, &dt.date,
            "日付が一致しない: {dt:?} -> {serialized:?} -> {parsed:?}"
        );
        match (&dt.time, &parsed_dt.time) {
            (Some(t1), Some(t2)) => {
                assert_eq!(t1.hour, t2.hour, "時が一致しない: {dt:?} -> {serialized:?}");
                assert_eq!(
                    t1.minute, t2.minute,
                    "分が一致しない: {dt:?} -> {serialized:?}"
                );
                assert_eq!(
                    t1.second, t2.second,
                    "秒が一致しない: {dt:?} -> {serialized:?}"
                );
                assert_eq!(
                    t1.nanosecond, t2.nanosecond,
                    "ナノ秒が一致しない: {dt:?} -> {serialized:?}"
                );
            }
            (None, None) => {}
            _ => panic!("時刻の有無が一致しない: {dt:?} -> {serialized:?} -> {parsed:?}"),
        }
        assert_eq!(
            &parsed_dt.offset, &dt.offset,
            "オフセットが一致しない: {dt:?} -> {serialized:?}"
        );
        Ok(())
    })?;
    Ok(())
}
