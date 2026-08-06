use pbt::bare_key_strategy;
use proptest::prelude::*;

proptest! {
    /// from_str の結果を to_string し、再度 from_str しても同一結果が得られる。
    #[test]
    fn parse_serialize_parse_roundtrip(table in pbt::table_strategy()) {
        let value = shiguredo_toml::Value::Table(table);
        let serialized = shiguredo_toml::to_string(&value).expect("シリアライズに成功するはず");
        let parsed1 = shiguredo_toml::from_str(&serialized).expect("TOML のパースに成功するはず");
        let re_serialized = shiguredo_toml::to_string(&shiguredo_toml::Value::Table(parsed1.clone()))
            .expect("シリアライズに成功するはず");
        let parsed2 = shiguredo_toml::from_str(&re_serialized).expect("TOML のパースに成功するはず");
        prop_assert_eq!(parsed1, parsed2);
    }

    /// 単純なキー値ペアの解析。
    #[test]
    fn parse_simple_key_value(
        key in bare_key_strategy(),
        val in any::<i64>(),
    ) {
        let input = format!("{key} = {val}");
        let table = shiguredo_toml::from_str(&input).expect("TOML のパースに成功するはず");
        prop_assert_eq!(table.get(&key).expect("キーは存在するはず").as_integer().expect("値は整数になるはず"), val);
    }

    /// bool 値の解析。
    #[test]
    fn parse_bool_value(
        key in bare_key_strategy(),
        b in any::<bool>(),
    ) {
        let input = format!("{key} = {b}");
        let table = shiguredo_toml::from_str(&input).expect("TOML のパースに成功するはず");
        prop_assert_eq!(table.get(&key).expect("キーは存在するはず").as_bool().expect("値はブール値になるはず"), b);
    }

    /// 空テーブルの解析と直列化。
    #[test]
    fn empty_table_roundtrip(_dummy in 0..1u8) {
        let input = "";
        let table = shiguredo_toml::from_str(input).expect("TOML のパースに成功するはず");
        prop_assert!(table.is_empty());
        let serialized = shiguredo_toml::to_string(&shiguredo_toml::Value::Table(table)).expect("シリアライズに成功するはず");
        prop_assert_eq!(serialized, "");
    }

    /// V1_1 でシリアライズした結果を V1_1 パーサで再パースしても同一結果が得られる。
    #[test]
    fn parse_serialize_parse_roundtrip_v1_1(table in pbt::table_strategy()) {
        let value = shiguredo_toml::Value::Table(table);
        let serialized = shiguredo_toml::to_string(&value).expect("シリアライズに成功するはず");
        let parsed1 = shiguredo_toml::from_str_with_version(&serialized, shiguredo_toml::TomlVersion::V1_1).expect("TOML のパースに成功するはず");
        let re_serialized = shiguredo_toml::to_string(&shiguredo_toml::Value::Table(parsed1.clone()))
            .expect("シリアライズに成功するはず");
        let parsed2 = shiguredo_toml::from_str_with_version(&re_serialized, shiguredo_toml::TomlVersion::V1_1).expect("TOML のパースに成功するはず");
        prop_assert_eq!(parsed1, parsed2);
    }
}
