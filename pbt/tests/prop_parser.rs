use pbt::{sample_bare_key, sample_table};

/// from_str の結果を to_string し、再度 from_str しても同一結果が得られる。
#[test]
fn parse_serialize_parse_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let table = sample_table(ctx);
        let value = shiguredo_toml::Value::Table(table);
        let serialized = shiguredo_toml::to_string(&value).expect("シリアライズに成功するはず");
        let parsed1 = shiguredo_toml::from_str(&serialized).expect("TOML のパースに成功するはず");
        let re_serialized =
            shiguredo_toml::to_string(&shiguredo_toml::Value::Table(parsed1.clone()))
                .expect("シリアライズに成功するはず");
        let parsed2 =
            shiguredo_toml::from_str(&re_serialized).expect("TOML のパースに成功するはず");
        assert_eq!(parsed1, parsed2, "パース結果が一致すること");
        Ok(())
    })?;
    Ok(())
}

/// 単純なキー値ペアの解析。
#[test]
fn parse_simple_key_value() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let key = sample_bare_key(ctx);
        let val = noprop::sample_i64(ctx);
        let input = format!("{key} = {val}");
        let table = shiguredo_toml::from_str(&input).expect("TOML のパースに成功するはず");
        assert_eq!(
            table
                .get(&key)
                .expect("キーは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            val,
            "整数の解析結果が一致すること"
        );
        Ok(())
    })?;
    Ok(())
}

/// bool 値の解析。
#[test]
fn parse_bool_value() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let key = sample_bare_key(ctx);
        let b = noprop::sample_bool(ctx);
        let input = format!("{key} = {b}");
        let table = shiguredo_toml::from_str(&input).expect("TOML のパースに成功するはず");
        assert_eq!(
            table
                .get(&key)
                .expect("キーは存在するはず")
                .as_bool()
                .expect("値はブール値になるはず"),
            b,
            "ブール値の解析結果が一致すること"
        );
        Ok(())
    })?;
    Ok(())
}

/// 空テーブルの解析と直列化。
///
/// 入力は常に空文字列で決定的なため、ケース数は 1 で十分である。
#[test]
fn empty_table_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(1, |_ctx| {
        let input = "";
        let table = shiguredo_toml::from_str(input).expect("TOML のパースに成功するはず");
        assert!(table.is_empty(), "空のテーブルになること");
        let serialized = shiguredo_toml::to_string(&shiguredo_toml::Value::Table(table))
            .expect("シリアライズに成功するはず");
        assert_eq!(serialized, "", "空テーブルの直列化結果は空文字列になること");
        Ok(())
    })?;
    Ok(())
}

/// to_string の結果を V1_1 パーサで再パースしても同一結果が得られる。
#[test]
fn parse_serialize_parse_roundtrip_v1_1() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let table = sample_table(ctx);
        let value = shiguredo_toml::Value::Table(table);
        let serialized = shiguredo_toml::to_string(&value).expect("シリアライズに成功するはず");
        let parsed1 =
            shiguredo_toml::from_str_with_version(&serialized, shiguredo_toml::TomlVersion::V1_1)
                .expect("TOML のパースに成功するはず");
        let re_serialized =
            shiguredo_toml::to_string(&shiguredo_toml::Value::Table(parsed1.clone()))
                .expect("シリアライズに成功するはず");
        let parsed2 = shiguredo_toml::from_str_with_version(
            &re_serialized,
            shiguredo_toml::TomlVersion::V1_1,
        )
        .expect("TOML のパースに成功するはず");
        assert_eq!(parsed1, parsed2, "パース結果が一致すること");
        Ok(())
    })?;
    Ok(())
}
