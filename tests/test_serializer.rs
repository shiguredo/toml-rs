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

mod datetime_validate {
    use super::*;
    use shiguredo_toml::{Date, Datetime, Offset, Time};

    /// パース経路で生成される valid な 4 バリアントの入力。
    /// シリアライズ結果の正準形の検証にも使う。
    const VALID_INPUTS: [&str; 4] = [
        "1979-05-27T07:32:00Z",
        "1979-05-27T07:32:00",
        "1979-05-27",
        "07:32:00",
    ];

    /// valid な Date。
    fn valid_date() -> Date {
        Date {
            year: 2024,
            month: 1,
            day: 1,
        }
    }

    /// valid な Time。
    fn valid_time() -> Time {
        Time {
            hour: 1,
            minute: 2,
            second: 3,
            nanosecond: 0,
        }
    }

    /// テーブルに包んだ無効な Datetime が to_string / to_string_pretty で
    /// Error::Serialize になることを検証する。
    fn assert_table_serialization_errors(dt: &Datetime) {
        let mut table = Table::new();
        table.insert("t".into(), Value::Datetime(dt.clone()));
        let value = Value::Table(table);

        let result = shiguredo_toml::to_string(&value);
        assert!(
            matches!(result, Err(shiguredo_toml::Error::Serialize { .. })),
            "to_string should fail for invalid datetime"
        );

        let result = shiguredo_toml::to_string_pretty(&value);
        assert!(
            matches!(result, Err(shiguredo_toml::Error::Serialize { .. })),
            "to_string_pretty should fail for invalid datetime"
        );
    }

    /// 単体の無効な Datetime が to_inline_string で Error::Serialize になる
    /// ことを検証する。
    fn assert_inline_serialization_errors(dt: &Datetime) {
        let result = shiguredo_toml::to_inline_string(&Value::Datetime(dt.clone()));
        assert!(
            matches!(result, Err(shiguredo_toml::Error::Serialize { .. })),
            "to_inline_string should fail for invalid datetime"
        );
    }

    /// 配列やテーブルにネストした無効な Datetime も Error::Serialize になる
    /// ことを検証する。
    fn assert_nested_serialization_errors(dt: &Datetime) {
        // 配列内
        let mut table = Table::new();
        table.insert(
            "arr".into(),
            Value::Array(vec![Value::Datetime(dt.clone())]),
        );
        let value = Value::Table(table);
        let result = shiguredo_toml::to_string(&value);
        assert!(
            matches!(result, Err(shiguredo_toml::Error::Serialize { .. })),
            "to_string should fail for invalid datetime in array"
        );
        let result = shiguredo_toml::to_string_pretty(&value);
        assert!(
            matches!(result, Err(shiguredo_toml::Error::Serialize { .. })),
            "to_string_pretty should fail for invalid datetime in array"
        );

        // サブテーブル内
        let mut table = Table::new();
        let mut inner = Table::new();
        inner.insert("t".into(), Value::Datetime(dt.clone()));
        table.insert("obj".into(), Value::Table(inner));
        let value = Value::Table(table);
        let result = shiguredo_toml::to_string(&value);
        assert!(
            matches!(result, Err(shiguredo_toml::Error::Serialize { .. })),
            "to_string should fail for invalid datetime in subtable"
        );
        let result = shiguredo_toml::to_string_pretty(&value);
        assert!(
            matches!(result, Err(shiguredo_toml::Error::Serialize { .. })),
            "to_string_pretty should fail for invalid datetime in subtable"
        );
    }

    /// 無効な Datetime（date / time 両方 None、offset なし）がエラーになる。
    #[test]
    fn missing_date_and_time_errors() {
        let dt = Datetime {
            date: None,
            time: None,
            offset: None,
        };
        assert_table_serialization_errors(&dt);
        assert_inline_serialization_errors(&dt);
    }

    /// Error::Serialize のメッセージに検証エラーのメッセージが
    /// 保持されることを検証する。
    #[test]
    fn error_message_is_preserved() {
        let dt = Datetime {
            date: None,
            time: None,
            offset: None,
        };
        let result = shiguredo_toml::to_inline_string(&Value::Datetime(dt));
        let error = result.expect_err("serialization should fail");
        let message = match error {
            shiguredo_toml::Error::Serialize { message } => message,
            other => panic!("expected Error::Serialize, got {other:?}"),
        };
        assert!(
            message.contains("date or a time"),
            "error message should contain the validation message, got: {message}"
        );
    }

    /// 無効な Datetime（date / time 両方 None、offset あり）がエラーになる。
    #[test]
    fn missing_date_and_time_with_offset_errors() {
        let dt = Datetime {
            date: None,
            time: None,
            offset: Some(Offset::Z),
        };
        assert_table_serialization_errors(&dt);
        assert_inline_serialization_errors(&dt);
    }

    /// 範囲外の Date（month 13）を含む Datetime がエラーになる。
    #[test]
    fn invalid_month_errors() {
        let dt = Datetime {
            date: Some(Date {
                month: 13,
                ..valid_date()
            }),
            time: Some(valid_time()),
            offset: None,
        };
        assert_table_serialization_errors(&dt);
        assert_inline_serialization_errors(&dt);
    }

    /// 範囲外の Date（day 40）を含む Datetime がエラーになる。
    #[test]
    fn invalid_day_errors() {
        let dt = Datetime {
            date: Some(Date {
                day: 40,
                ..valid_date()
            }),
            time: Some(valid_time()),
            offset: None,
        };
        assert_table_serialization_errors(&dt);
        assert_inline_serialization_errors(&dt);
    }

    /// 範囲外の Date（year 10000）を含む Datetime がエラーになる。
    #[test]
    fn invalid_year_errors() {
        let dt = Datetime {
            date: Some(Date {
                year: 10000,
                ..valid_date()
            }),
            time: Some(valid_time()),
            offset: None,
        };
        assert_table_serialization_errors(&dt);
        assert_inline_serialization_errors(&dt);
    }

    /// 範囲外の Time（hour 24）を含む Datetime がエラーになる。
    #[test]
    fn invalid_hour_errors() {
        let dt = Datetime {
            date: Some(valid_date()),
            time: Some(Time {
                hour: 24,
                ..valid_time()
            }),
            offset: None,
        };
        assert_table_serialization_errors(&dt);
        assert_inline_serialization_errors(&dt);
    }

    /// 範囲外の Offset（minutes 1440）を含む Datetime がエラーになる。
    #[test]
    fn invalid_offset_errors() {
        let dt = Datetime {
            date: Some(valid_date()),
            time: Some(valid_time()),
            offset: Some(Offset::Custom { minutes: 1440 }),
        };
        assert_table_serialization_errors(&dt);
        assert_inline_serialization_errors(&dt);
    }

    /// date のみ + offset の組み合わせはオフセットが黙って破棄されるため
    /// エラーになる。
    #[test]
    fn date_only_with_offset_errors() {
        let dt = Datetime {
            date: Some(valid_date()),
            time: None,
            offset: Some(Offset::Z),
        };
        assert_table_serialization_errors(&dt);
        assert_inline_serialization_errors(&dt);
    }

    /// time のみ + offset の組み合わせはオフセットが黙って破棄されるため
    /// エラーになる。
    #[test]
    fn time_only_with_offset_errors() {
        let dt = Datetime {
            date: None,
            time: Some(valid_time()),
            offset: Some(Offset::Z),
        };
        assert_table_serialization_errors(&dt);
        assert_inline_serialization_errors(&dt);
    }

    /// 配列やサブテーブルにネストした無効な Datetime もエラーになる。
    #[test]
    fn nested_invalid_datetime_errors() {
        let dt = Datetime {
            date: None,
            time: None,
            offset: Some(Offset::Z),
        };
        assert_nested_serialization_errors(&dt);
    }

    /// パース経路で生成される valid な 4 バリアントのシリアライズ結果が
    /// 入力と同一の正準形になり、再パースで値が一致することを検証する。
    #[test]
    fn valid_datetime_roundtrip() {
        for input in VALID_INPUTS {
            let parsed =
                shiguredo_toml::from_str(&format!("t = {input}")).expect("TOML should parse");
            let serialized = shiguredo_toml::to_string(&Value::Table(parsed.clone()))
                .expect("serialization should succeed");
            // シリアライズ結果は入力の正準形と同一である
            assert_eq!(serialized, format!("t = {input}\n"));
            let reparsed = shiguredo_toml::from_str(&serialized).expect("TOML should reparse");
            assert_eq!(parsed, reparsed);
        }
    }

    /// valid な 4 バリアントの単体 Datetime が to_inline_string で
    /// 入力と同じ表現で出力されることを検証する。
    #[test]
    fn valid_datetime_inline_ok() {
        for input in VALID_INPUTS {
            let parsed =
                shiguredo_toml::from_str(&format!("t = {input}")).expect("TOML should parse");
            let dt = parsed["t"]
                .as_datetime()
                .expect("value should be a datetime");
            let result = shiguredo_toml::to_inline_string(&Value::Datetime(dt.clone()))
                .expect("inline serialization should succeed");
            assert_eq!(result, input);
        }
    }
}
