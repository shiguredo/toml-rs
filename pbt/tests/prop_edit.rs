use pbt::sample_bare_key;
use shiguredo_toml::{Document, Value};

/// 既存キーと必ず異なる新規キーのペアを生成する。
///
/// 既存キーと新規キーが同一だと `set_path` による挿入が既存値の置換になり、
/// 新規キーの挿入を検証するテストの意図を外れるため、末尾に 1 文字足して
/// 必ず異なるキーを生成する。
///
/// 編集テスト固有の制約（既存キーと新規キーの不一致）を表現するヘルパーのため、
/// 汎用の生成ヘルパーを集約する `pbt/src/lib.rs` には置かず、テストファイル内に置く。
fn sample_key_pair(ctx: &mut noprop::TestCaseContext) -> (String, String) {
    let existing_key = sample_bare_key(ctx);
    let mut new_key = existing_key.clone();
    new_key.push('_');
    (existing_key, new_key)
}

/// 既存キーの値を置換しても TOML として再解析可能で、置換値が反映される。
#[test]
fn replace_existing_scalar_value() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let key = sample_bare_key(ctx);
        let old = noprop::sample_i64(ctx);
        let new = noprop::sample_i64(ctx);
        let input = format!("{key} = {old}\n");
        let mut doc = Document::parse(&input).expect("TOML のパースに成功するはず");

        doc.set_path(&key, Value::Integer(new))
            .expect("編集に成功するはず");

        let parsed = shiguredo_toml::from_str(doc.as_str()).expect("TOML のパースに成功するはず");
        assert_eq!(
            parsed[&key].as_integer().expect("値は整数になるはず"),
            new,
            "置換後の値が一致すること"
        );
        Ok(())
    })?;
    Ok(())
}

/// 新規キーを挿入後、get_path で取得できる。
#[test]
fn insert_then_get_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let (existing_key, new_key) = sample_key_pair(ctx);
        let existing_val = noprop::sample_i64(ctx);
        let new_val = noprop::sample_i64(ctx);

        let input = format!("{existing_key} = {existing_val}\n");
        let mut doc = Document::parse(&input).expect("TOML のパースに成功するはず");

        doc.set_path(&new_key, Value::Integer(new_val))
            .expect("編集に成功するはず");

        // 挿入したキーが取得できる
        assert_eq!(
            doc.get_path(&new_key)
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            new_val,
            "挿入したキーの値が一致すること"
        );
        // 既存キーも保持される
        assert_eq!(
            doc.get_path(&existing_key)
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            existing_val,
            "既存キーの値が保持されること"
        );
        Ok(())
    })?;
    Ok(())
}

/// 挿入後の as_str() が有効な TOML として再パース可能。
#[test]
fn insert_produces_valid_toml() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let (existing_key, new_key) = sample_key_pair(ctx);
        let existing_val = noprop::sample_i64(ctx);
        let new_val = noprop::sample_i64(ctx);

        let input = format!("{existing_key} = {existing_val}\n");
        let mut doc = Document::parse(&input).expect("TOML のパースに成功するはず");

        doc.set_path(&new_key, Value::Integer(new_val))
            .expect("編集に成功するはず");

        // 出力が有効な TOML であることを検証する
        let parsed = shiguredo_toml::from_str(doc.as_str()).expect("TOML のパースに成功するはず");
        assert_eq!(
            parsed[&new_key].as_integer().expect("値は整数になるはず"),
            new_val,
            "挿入したキーの値が一致すること"
        );
        assert_eq!(
            parsed[&existing_key]
                .as_integer()
                .expect("値は整数になるはず"),
            existing_val,
            "既存キーの値が一致すること"
        );
        Ok(())
    })?;
    Ok(())
}
