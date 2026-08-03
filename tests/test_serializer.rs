use shiguredo_toml::{Table, Value};

mod basic {
    use super::*;

    #[test]
    fn empty_table() {
        let table = Table::new();
        let value = Value::Table(table);
        let s = shiguredo_toml::to_string(&value).expect("serialization should succeed");
        assert_eq!(s, "");
    }

    #[test]
    fn simple_key_value() {
        let mut table = Table::new();
        table.insert("key".into(), Value::String("value".into()));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert_eq!(s, "key = \"value\"\n");
    }

    #[test]
    fn integer_value() {
        let mut table = Table::new();
        table.insert("a".into(), Value::Integer(42));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert_eq!(s, "a = 42\n");
    }

    #[test]
    fn float_value() {
        let mut table = Table::new();
        table.insert("a".into(), Value::Float(2.72));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert!(s.starts_with("a = 2.72"));
    }

    #[test]
    fn float_integer_like() {
        let mut table = Table::new();
        table.insert("a".into(), Value::Float(1.0));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        // 整数っぽい浮動小数点数でも .0 が付く
        assert!(s.contains('.'));
    }

    #[test]
    fn float_inf() {
        let mut table = Table::new();
        table.insert("a".into(), Value::Float(f64::INFINITY));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert_eq!(s, "a = inf\n");
    }

    #[test]
    fn float_neg_inf() {
        let mut table = Table::new();
        table.insert("a".into(), Value::Float(f64::NEG_INFINITY));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert_eq!(s, "a = -inf\n");
    }

    #[test]
    fn float_nan() {
        let mut table = Table::new();
        table.insert("a".into(), Value::Float(f64::NAN));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert_eq!(s, "a = nan\n");
    }

    #[test]
    fn bool_value() {
        let mut table = Table::new();
        table.insert("a".into(), Value::Boolean(true));
        table.insert("b".into(), Value::Boolean(false));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert!(s.contains("a = true"));
        assert!(s.contains("b = false"));
    }

    #[test]
    fn string_escape() {
        let mut table = Table::new();
        table.insert("a".into(), Value::String("hello\nworld".into()));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert!(s.contains("\\n"));
    }

    #[test]
    fn string_with_quotes() {
        let mut table = Table::new();
        table.insert("a".into(), Value::String("say \"hi\"".into()));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert!(s.contains("\\\""));
    }

    #[test]
    fn string_with_backslash() {
        let mut table = Table::new();
        table.insert("a".into(), Value::String("C:\\path".into()));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert!(s.contains("\\\\"));
    }
}

mod structure {
    use super::*;

    #[test]
    fn sub_table() {
        let mut inner = Table::new();
        inner.insert("key".into(), Value::Integer(1));
        let mut table = Table::new();
        table.insert("section".into(), Value::Table(inner));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert!(s.contains("[section]"));
        assert!(s.contains("key = 1"));
    }

    #[test]
    fn array_of_tables() {
        let mut item1 = Table::new();
        item1.insert("name".into(), Value::String("a".into()));
        let mut item2 = Table::new();
        item2.insert("name".into(), Value::String("b".into()));
        let mut table = Table::new();
        table.insert(
            "items".into(),
            Value::Array(vec![Value::Table(item1), Value::Table(item2)]),
        );
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert!(s.contains("[[items]]"));
        assert!(s.contains("name = \"a\""));
        assert!(s.contains("name = \"b\""));
    }

    #[test]
    fn inline_array() {
        let mut table = Table::new();
        table.insert(
            "arr".into(),
            Value::Array(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ]),
        );
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert_eq!(s, "arr = [1, 2, 3]\n");
    }

    #[test]
    fn empty_array() {
        let mut table = Table::new();
        table.insert("arr".into(), Value::Array(vec![]));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert_eq!(s, "arr = []\n");
    }

    #[test]
    fn key_quoting() {
        let mut table = Table::new();
        table.insert("hello world".into(), Value::Integer(1));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert!(s.contains("\"hello world\""));
    }

    #[test]
    fn empty_key_quoting() {
        let mut table = Table::new();
        table.insert("".into(), Value::Integer(1));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("serialization should succeed");
        assert!(s.contains("\"\""));
    }

    #[test]
    fn non_table_top_level_error() {
        let result = shiguredo_toml::to_string(&Value::Integer(42));
        assert!(result.is_err());
    }
}

mod pretty {
    use super::*;

    #[test]
    fn blank_lines_between_tables() {
        let mut inner1 = Table::new();
        inner1.insert("a".into(), Value::Integer(1));
        let mut inner2 = Table::new();
        inner2.insert("b".into(), Value::Integer(2));
        let mut table = Table::new();
        table.insert("x".into(), Value::Table(inner1));
        table.insert("y".into(), Value::Table(inner2));
        let s = shiguredo_toml::to_string_pretty(&Value::Table(table))
            .expect("serialization should succeed");
        // pretty モードではテーブル間に空行
        assert!(s.contains("\n\n"));
    }
}
