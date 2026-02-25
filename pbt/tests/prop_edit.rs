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
}
