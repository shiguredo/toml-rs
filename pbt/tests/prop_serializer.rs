use pbt::{bare_key_strategy, safe_string_strategy};
use proptest::prelude::*;

proptest! {
    /// 文字列値のラウンドトリップ。
    #[test]
    fn string_roundtrip(
        key in bare_key_strategy(),
        value in safe_string_strategy(),
    ) {
        let table = shiguredo_toml::from_str(&format!("{key} = \"\"")).unwrap();
        // まず空文字列をパースして構造確認
        prop_assert!(table.get(&key).unwrap().is_str());

        // 実際の値を含むテーブルをプログラムで構築して直列化
        let mut table = shiguredo_toml::Table::new();
        table.insert(key.clone(), shiguredo_toml::Value::String(value.clone()));
        let serialized = shiguredo_toml::to_string(&shiguredo_toml::Value::Table(table)).unwrap();
        let parsed = shiguredo_toml::from_str(&serialized).unwrap();
        prop_assert_eq!(parsed.get(&key).unwrap().as_str().unwrap(), &value);
    }

    /// 浮動小数点数のラウンドトリップ（有限値のみ）。
    #[test]
    fn float_roundtrip(
        key in bare_key_strategy(),
        f in (-1e100f64..1e100),
    ) {
        let mut table = shiguredo_toml::Table::new();
        table.insert(key.clone(), shiguredo_toml::Value::Float(f));
        let serialized = shiguredo_toml::to_string(&shiguredo_toml::Value::Table(table)).unwrap();
        let parsed = shiguredo_toml::from_str(&serialized).unwrap();
        let parsed_f = parsed.get(&key).unwrap().as_float().unwrap();
        prop_assert!((parsed_f - f).abs() < 1e-10 || parsed_f == f,
            "Float mismatch: {f} -> {parsed_f}");
    }

    /// Datetime のラウンドトリップ。
    #[test]
    fn datetime_roundtrip(
        key in bare_key_strategy(),
        dt in pbt::datetime_strategy(),
    ) {
        let dt_str = dt.to_string();
        if dt_str.is_empty() {
            return Ok(());
        }
        let mut table = shiguredo_toml::Table::new();
        table.insert(key.clone(), shiguredo_toml::Value::Datetime(dt.clone()));
        let serialized = shiguredo_toml::to_string(&shiguredo_toml::Value::Table(table)).unwrap();
        let parsed = shiguredo_toml::from_str(&serialized).unwrap();
        let parsed_dt = parsed.get(&key).unwrap().as_datetime().unwrap();
        prop_assert_eq!(&parsed_dt.date, &dt.date);
        match (&dt.time, &parsed_dt.time) {
            (Some(t1), Some(t2)) => {
                prop_assert_eq!(t1.hour, t2.hour);
                prop_assert_eq!(t1.minute, t2.minute);
                prop_assert_eq!(t1.second, t2.second);
                prop_assert_eq!(t1.nanosecond, t2.nanosecond);
            }
            (None, None) => {}
            _ => prop_assert!(false, "Time presence mismatch"),
        }
        prop_assert_eq!(&parsed_dt.offset, &dt.offset);
    }
}
