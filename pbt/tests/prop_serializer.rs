use pbt::{sample_bare_key, sample_datetime, sample_safe_string};

/// 文字列値のラウンドトリップ。
#[test]
fn string_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let key = sample_bare_key(ctx);
        let value = sample_safe_string(ctx);
        let table = shiguredo_toml::from_str(&format!("{key} = \"\""))
            .expect("TOML のパースに成功するはず");
        // まず空文字列をパースして構造確認
        assert!(
            table.get(&key).expect("キーは存在するはず").is_str(),
            "キーは文字列値になること"
        );

        // 実際の値を含むテーブルをプログラムで構築して直列化
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
            "文字列値が一致すること"
        );
        Ok(())
    })?;
    Ok(())
}

/// 浮動小数点数のラウンドトリップ（有限値のみ）。
#[test]
fn float_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let key = sample_bare_key(ctx);
        let f = noprop::sample_f64_in(ctx, -1e100, 1e100);
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
            "浮動小数点数の不一致: {f} -> {parsed_f}"
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
        assert_eq!(&parsed_dt.date, &dt.date, "日付が一致すること");
        match (&dt.time, &parsed_dt.time) {
            (Some(t1), Some(t2)) => {
                assert_eq!(t1.hour, t2.hour, "時が一致すること");
                assert_eq!(t1.minute, t2.minute, "分が一致すること");
                assert_eq!(t1.second, t2.second, "秒が一致すること");
                assert_eq!(t1.nanosecond, t2.nanosecond, "ナノ秒が一致すること");
            }
            (None, None) => {}
            _ => panic!("時刻の有無が一致しない"),
        }
        assert_eq!(&parsed_dt.offset, &dt.offset, "オフセットが一致すること");
        Ok(())
    })?;
    Ok(())
}
