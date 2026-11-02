use shiguredo_toml::{Date, Datetime, Offset, Table, Time, Value};

mod simple_key_values {
    use super::*;

    #[test]
    fn all_scalar_types() {
        let mut table = Table::new();
        table.insert("bool_val".into(), Value::Boolean(true));
        table.insert("float_val".into(), Value::Float(3.15));
        table.insert("int_val".into(), Value::Integer(42));
        table.insert("str_val".into(), Value::String("hello".into()));
        table.insert(
            "dt_val".into(),
            Value::Datetime(Datetime {
                date: Some(Date {
                    year: 1979,
                    month: 5,
                    day: 27,
                }),
                time: Some(Time {
                    hour: 7,
                    minute: 32,
                    second: 0,
                    nanosecond: 0,
                }),
                offset: Some(Offset::Z),
            }),
        );
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn special_floats() {
        let mut table = Table::new();
        table.insert("inf_val".into(), Value::Float(f64::INFINITY));
        table.insert("nan_val".into(), Value::Float(f64::NAN));
        table.insert("neg_inf_val".into(), Value::Float(f64::NEG_INFINITY));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn integer_like_float() {
        let mut table = Table::new();
        table.insert("a".into(), Value::Float(1.0));
        table.insert("b".into(), Value::Float(0.0));
        table.insert("c".into(), Value::Float(-0.0));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }
}

mod sub_tables {
    use super::*;

    #[test]
    fn single_sub_table() {
        let mut inner = Table::new();
        inner.insert("host".into(), Value::String("localhost".into()));
        inner.insert("port".into(), Value::Integer(8080));
        let mut table = Table::new();
        table.insert("server".into(), Value::Table(inner));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn nested_sub_tables() {
        let mut tls = Table::new();
        tls.insert("enabled".into(), Value::Boolean(true));
        let mut server = Table::new();
        server.insert("host".into(), Value::String("localhost".into()));
        server.insert("tls".into(), Value::Table(tls));
        let mut table = Table::new();
        table.insert("server".into(), Value::Table(server));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn multiple_sub_tables() {
        let mut db = Table::new();
        db.insert("name".into(), Value::String("mydb".into()));
        let mut server = Table::new();
        server.insert("port".into(), Value::Integer(8080));
        let mut table = Table::new();
        table.insert("database".into(), Value::Table(db));
        table.insert("server".into(), Value::Table(server));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn table_with_inline_values_and_sub_table() {
        let mut sub = Table::new();
        sub.insert("c".into(), Value::Integer(3));
        let mut table = Table::new();
        table.insert("a".into(), Value::Integer(1));
        table.insert("b".into(), Value::Integer(2));
        table.insert("sub".into(), Value::Table(sub));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }
}

mod array_of_tables {
    use super::*;

    #[test]
    fn basic_array_of_tables() {
        let mut item1 = Table::new();
        item1.insert("name".into(), Value::String("alpha".into()));
        let mut item2 = Table::new();
        item2.insert("name".into(), Value::String("beta".into()));
        let mut table = Table::new();
        table.insert(
            "items".into(),
            Value::Array(vec![Value::Table(item1), Value::Table(item2)]),
        );
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn array_of_tables_with_sub_tables() {
        let mut sub = Table::new();
        sub.insert("key".into(), Value::String("val".into()));
        let mut item = Table::new();
        item.insert("name".into(), Value::String("x".into()));
        item.insert("details".into(), Value::Table(sub));
        let mut table = Table::new();
        table.insert("items".into(), Value::Array(vec![Value::Table(item)]));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }
}

mod pretty_format {
    use super::*;

    #[test]
    fn blank_lines_between_tables() {
        let mut a = Table::new();
        a.insert("x".into(), Value::Integer(1));
        let mut b = Table::new();
        b.insert("y".into(), Value::Integer(2));
        let mut table = Table::new();
        table.insert("a".into(), Value::Table(a));
        table.insert("b".into(), Value::Table(b));
        let s = shiguredo_toml::to_string_pretty(&Value::Table(table))
            .expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn pretty_with_top_level_keys() {
        let mut sub = Table::new();
        sub.insert("inner".into(), Value::Integer(1));
        let mut table = Table::new();
        table.insert("top".into(), Value::String("value".into()));
        table.insert("section".into(), Value::Table(sub));
        let s = shiguredo_toml::to_string_pretty(&Value::Table(table))
            .expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn pretty_array_of_tables() {
        let mut item1 = Table::new();
        item1.insert("a".into(), Value::Integer(1));
        let mut item2 = Table::new();
        item2.insert("a".into(), Value::Integer(2));
        let mut table = Table::new();
        table.insert(
            "items".into(),
            Value::Array(vec![Value::Table(item1), Value::Table(item2)]),
        );
        let s = shiguredo_toml::to_string_pretty(&Value::Table(table))
            .expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }
}

mod key_quoting {
    use super::*;

    #[test]
    fn keys_requiring_quotes() {
        let mut table = Table::new();
        table.insert("".into(), Value::Integer(1));
        table.insert("hello world".into(), Value::Integer(2));
        table.insert("key=val".into(), Value::Integer(3));
        table.insert("normal".into(), Value::Integer(4));
        table.insert("with-dash".into(), Value::Integer(5));
        table.insert("with_underscore".into(), Value::Integer(6));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn quoted_key_in_table_header() {
        let mut inner = Table::new();
        inner.insert("key".into(), Value::Integer(1));
        let mut table = Table::new();
        table.insert("has space".into(), Value::Table(inner));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }
}

mod string_escaping {
    use super::*;

    #[test]
    fn escape_sequences() {
        let mut table = Table::new();
        table.insert("backslash".into(), Value::String("a\\b".into()));
        table.insert("newline".into(), Value::String("a\nb".into()));
        table.insert("quotes".into(), Value::String("say \"hi\"".into()));
        table.insert("tab".into(), Value::String("a\tb".into()));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn control_characters() {
        let mut table = Table::new();
        table.insert("backspace".into(), Value::String("a\u{0008}b".into()));
        table.insert("formfeed".into(), Value::String("a\u{000C}b".into()));
        table.insert("cr".into(), Value::String("a\rb".into()));
        table.insert("null".into(), Value::String("a\u{0000}b".into()));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }
}

mod inline_structures {
    use super::*;

    #[test]
    fn inline_array_various() {
        let mut table = Table::new();
        table.insert("empty".into(), Value::Array(vec![]));
        table.insert(
            "ints".into(),
            Value::Array(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ]),
        );
        table.insert(
            "mixed".into(),
            Value::Array(vec![
                Value::Integer(1),
                Value::String("two".into()),
                Value::Boolean(true),
            ]),
        );
        table.insert(
            "nested".into(),
            Value::Array(vec![
                Value::Array(vec![Value::Integer(1), Value::Integer(2)]),
                Value::Array(vec![Value::Integer(3), Value::Integer(4)]),
            ]),
        );
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }

    #[test]
    fn inline_table_in_value() {
        // インラインテーブルを含む配列（配列テーブルにならないケース）はない
        // ここではトップレベルに値とサブテーブルが混在するケースを確認
        let mut inner = Table::new();
        inner.insert("x".into(), Value::Integer(1));
        inner.insert("y".into(), Value::Integer(2));
        let mut table = Table::new();
        table.insert(
            "points".into(),
            Value::Array(vec![Value::Integer(1), Value::Integer(2)]),
        );
        table.insert("origin".into(), Value::Table(inner));
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }
}

mod datetime_output {
    use super::*;

    #[test]
    fn all_datetime_variants() {
        let mut table = Table::new();
        table.insert(
            "local_date".into(),
            Value::Datetime(Datetime {
                date: Some(Date {
                    year: 2024,
                    month: 1,
                    day: 15,
                }),
                time: None,
                offset: None,
            }),
        );
        table.insert(
            "local_datetime".into(),
            Value::Datetime(Datetime {
                date: Some(Date {
                    year: 2024,
                    month: 1,
                    day: 15,
                }),
                time: Some(Time {
                    hour: 10,
                    minute: 30,
                    second: 0,
                    nanosecond: 0,
                }),
                offset: None,
            }),
        );
        table.insert(
            "local_time".into(),
            Value::Datetime(Datetime {
                date: None,
                time: Some(Time {
                    hour: 10,
                    minute: 30,
                    second: 45,
                    nanosecond: 123000000,
                }),
                offset: None,
            }),
        );
        table.insert(
            "offset_datetime".into(),
            Value::Datetime(Datetime {
                date: Some(Date {
                    year: 2024,
                    month: 1,
                    day: 15,
                }),
                time: Some(Time {
                    hour: 10,
                    minute: 30,
                    second: 0,
                    nanosecond: 0,
                }),
                offset: Some(Offset::Custom { minutes: 540 }),
            }),
        );
        table.insert(
            "utc_datetime".into(),
            Value::Datetime(Datetime {
                date: Some(Date {
                    year: 2024,
                    month: 1,
                    day: 15,
                }),
                time: Some(Time {
                    hour: 10,
                    minute: 30,
                    second: 0,
                    nanosecond: 0,
                }),
                offset: Some(Offset::Z),
            }),
        );
        let s =
            shiguredo_toml::to_string(&Value::Table(table)).expect("シリアライズに成功するはず");
        insta::assert_snapshot!(s);
    }
}
