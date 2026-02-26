use pbt::bare_key_strategy;
use proptest::prelude::*;
use shiguredo_toml::{Document, Value};

proptest! {
    /// 既存キーの値を置換しても TOML として再解析可能で、置換値が反映される。
    #[test]
    fn replace_existing_scalar_value(
        key in bare_key_strategy(),
        old in any::<i64>(),
        new in any::<i64>(),
    ) {
        let input = format!("{key} = {old}\n");
        let mut doc = Document::parse(&input).unwrap();

        doc.set_path(&key, Value::Integer(new)).unwrap();

        let parsed = shiguredo_toml::from_str(doc.as_str()).unwrap();
        prop_assert_eq!(parsed[&key].as_integer().unwrap(), new);
    }

    /// 新規キーを挿入後、get_path で取得できる。
    #[test]
    fn insert_then_get_roundtrip(
        existing_key in bare_key_strategy(),
        new_key in bare_key_strategy(),
        existing_val in any::<i64>(),
        new_val in any::<i64>(),
    ) {
        // 既存キーと新規キーが異なる場合のみテストする
        prop_assume!(existing_key != new_key);

        let input = format!("{existing_key} = {existing_val}\n");
        let mut doc = Document::parse(&input).unwrap();

        doc.set_path(&new_key, Value::Integer(new_val)).unwrap();

        // 挿入したキーが取得できる
        prop_assert_eq!(
            doc.get_path(&new_key).unwrap().as_integer().unwrap(),
            new_val
        );
        // 既存キーも保持される
        prop_assert_eq!(
            doc.get_path(&existing_key).unwrap().as_integer().unwrap(),
            existing_val
        );
    }

    /// 挿入後の as_str() が有効な TOML として再パース可能。
    #[test]
    fn insert_produces_valid_toml(
        existing_key in bare_key_strategy(),
        new_key in bare_key_strategy(),
        existing_val in any::<i64>(),
        new_val in any::<i64>(),
    ) {
        prop_assume!(existing_key != new_key);

        let input = format!("{existing_key} = {existing_val}\n");
        let mut doc = Document::parse(&input).unwrap();

        doc.set_path(&new_key, Value::Integer(new_val)).unwrap();

        // 出力が有効な TOML であることを検証する
        let parsed = shiguredo_toml::from_str(doc.as_str()).unwrap();
        prop_assert_eq!(parsed[&new_key].as_integer().unwrap(), new_val);
        prop_assert_eq!(parsed[&existing_key].as_integer().unwrap(), existing_val);
    }
}
