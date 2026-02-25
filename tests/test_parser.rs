mod parse_error {
    #[test]
    fn empty_input() {
        let result = shiguredo_toml::from_str("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn unclosed_basic_string() {
        let result = shiguredo_toml::from_str("key = \"unclosed");
        assert!(result.is_err());
    }

    #[test]
    fn unclosed_literal_string() {
        let result = shiguredo_toml::from_str("key = 'unclosed");
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_key() {
        let result = shiguredo_toml::from_str("a = 1\na = 2");
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_table() {
        let result = shiguredo_toml::from_str("[a]\n[a]");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_bare_key_chars() {
        let result = shiguredo_toml::from_str("bad key = 1");
        assert!(result.is_err());
    }

    #[test]
    fn leading_zero_integer() {
        let result = shiguredo_toml::from_str("a = 01");
        assert!(result.is_err());
    }

    #[test]
    fn underscore_start_integer() {
        let result = shiguredo_toml::from_str("a = _1");
        assert!(result.is_err());
    }

    #[test]
    fn underscore_end_integer() {
        let result = shiguredo_toml::from_str("a = 1_");
        assert!(result.is_err());
    }

    #[test]
    fn double_underscore_integer() {
        let result = shiguredo_toml::from_str("a = 1__0");
        assert!(result.is_err());
    }

    #[test]
    fn inline_table_trailing_comma_v1() {
        let result = shiguredo_toml::from_str("a = {b = 1,}");
        assert!(result.is_err());
    }

    #[test]
    fn inline_table_newline_v1() {
        let result = shiguredo_toml::from_str("a = {b = 1,\nc = 2}");
        assert!(result.is_err());
    }

    #[test]
    fn multiline_key_not_allowed() {
        let result = shiguredo_toml::from_str("\"\"\"key\"\"\" = 1");
        assert!(result.is_err());
    }

    #[test]
    fn redefine_inline_table() {
        let result = shiguredo_toml::from_str("a = {b = 1}\n[a]");
        assert!(result.is_err());
    }

    #[test]
    fn add_to_inline_table() {
        let result = shiguredo_toml::from_str("a = {b = 1}\n[a]\nc = 2");
        assert!(result.is_err());
    }

    #[test]
    fn control_char_in_comment() {
        let result = shiguredo_toml::from_str("# \x01");
        assert!(result.is_err());
    }

    #[test]
    fn control_char_in_basic_string() {
        let result = shiguredo_toml::from_str("a = \"\x01\"");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_escape_sequence() {
        let result = shiguredo_toml::from_str("a = \"\\x41\"");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_unicode_escape() {
        let result = shiguredo_toml::from_str("a = \"\\uD800\"");
        assert!(result.is_err());
    }

    #[test]
    fn newline_in_basic_string() {
        let result = shiguredo_toml::from_str("a = \"hello\nworld\"");
        assert!(result.is_err());
    }

    #[test]
    fn newline_in_literal_string() {
        let result = shiguredo_toml::from_str("a = 'hello\nworld'");
        assert!(result.is_err());
    }

    #[test]
    fn redefine_table_defined_by_dotted_key() {
        // v1.0.0: dotted keys で定義されたテーブルは [header] で再定義できない
        let input = "[fruit]\napple.color = \"red\"\n[fruit.apple]\ntexture = \"smooth\"\n";
        let result = shiguredo_toml::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn table_header_conflicts_with_array_of_tables() {
        // v1.0.0: 配列テーブルと同名の通常テーブルは定義できない
        let input = "[[fruits]]\nname = \"apple\"\n[fruits]\ncolor = \"red\"\n";
        let result = shiguredo_toml::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn table_header_conflicts_with_nested_array_of_tables() {
        // v1.0.0: 既に配列テーブルとして確立したパスに通常テーブルを定義できない
        let input = "[[fruits]]\nname = \"apple\"\n[[fruits.varieties]]\nname = \"red delicious\"\n[fruits.varieties]\nname = \"granny smith\"\n";
        let result = shiguredo_toml::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_newline_cr_only() {
        // v1.0.0: 改行は LF または CRLF のみ
        let input = "\ra = 1\n";
        let result = shiguredo_toml::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn inline_table_direct_key_then_dotted_key_is_invalid() {
        let input = "tab = { inner = { dog = \"best\" }, inner.cat = \"worst\" }";
        let result = shiguredo_toml::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn dotted_key_cannot_extend_explicit_table() {
        let input = "[a.b.c]\nz = 9\n[a]\nb.c.t = \"no\"\n";
        let result = shiguredo_toml::from_str(input);
        assert!(result.is_err());
    }
}

mod parse_success {
    use shiguredo_toml::Value;

    #[test]
    fn basic_string() {
        let t = shiguredo_toml::from_str(r#"a = "hello""#).unwrap();
        assert_eq!(t["a"].as_str().unwrap(), "hello");
    }

    #[test]
    fn basic_string_escapes() {
        let t = shiguredo_toml::from_str(r#"a = "hello\nworld""#).unwrap();
        assert_eq!(t["a"].as_str().unwrap(), "hello\nworld");
    }

    #[test]
    fn literal_string() {
        let t = shiguredo_toml::from_str(r#"a = 'C:\Users\path'"#).unwrap();
        assert_eq!(t["a"].as_str().unwrap(), "C:\\Users\\path");
    }

    #[test]
    fn multiline_basic_string() {
        let t = shiguredo_toml::from_str("a = \"\"\"\nhello\nworld\"\"\"").unwrap();
        assert_eq!(t["a"].as_str().unwrap(), "hello\nworld");
    }

    #[test]
    fn multiline_literal_string() {
        let t = shiguredo_toml::from_str("a = '''\nhello\nworld'''").unwrap();
        assert_eq!(t["a"].as_str().unwrap(), "hello\nworld");
    }

    #[test]
    fn integer_decimal() {
        let t = shiguredo_toml::from_str("a = 42").unwrap();
        assert_eq!(t["a"].as_integer().unwrap(), 42);
    }

    #[test]
    fn integer_negative() {
        let t = shiguredo_toml::from_str("a = -42").unwrap();
        assert_eq!(t["a"].as_integer().unwrap(), -42);
    }

    #[test]
    fn integer_positive_sign() {
        let t = shiguredo_toml::from_str("a = +42").unwrap();
        assert_eq!(t["a"].as_integer().unwrap(), 42);
    }

    #[test]
    fn integer_hex() {
        let t = shiguredo_toml::from_str("a = 0xDEAD_BEEF").unwrap();
        assert_eq!(t["a"].as_integer().unwrap(), 0xDEADBEEF);
    }

    #[test]
    fn integer_oct() {
        let t = shiguredo_toml::from_str("a = 0o755").unwrap();
        assert_eq!(t["a"].as_integer().unwrap(), 0o755);
    }

    #[test]
    fn integer_bin() {
        let t = shiguredo_toml::from_str("a = 0b1101").unwrap();
        assert_eq!(t["a"].as_integer().unwrap(), 0b1101);
    }

    #[test]
    fn integer_underscore() {
        let t = shiguredo_toml::from_str("a = 1_000_000").unwrap();
        assert_eq!(t["a"].as_integer().unwrap(), 1_000_000);
    }

    #[test]
    fn float_basic() {
        let t = shiguredo_toml::from_str("a = 2.72").unwrap();
        let f = t["a"].as_float().unwrap();
        assert!((f - 2.72).abs() < 1e-10);
    }

    #[test]
    fn float_exponent() {
        let t = shiguredo_toml::from_str("a = 5e+22").unwrap();
        let f = t["a"].as_float().unwrap();
        assert!((f - 5e22).abs() < 1e12);
    }

    #[test]
    fn float_inf() {
        let t = shiguredo_toml::from_str("a = inf").unwrap();
        assert!(t["a"].as_float().unwrap().is_infinite());
        assert!(t["a"].as_float().unwrap().is_sign_positive());
    }

    #[test]
    fn float_neg_inf() {
        let t = shiguredo_toml::from_str("a = -inf").unwrap();
        assert!(t["a"].as_float().unwrap().is_infinite());
        assert!(t["a"].as_float().unwrap().is_sign_negative());
    }

    #[test]
    fn float_nan() {
        let t = shiguredo_toml::from_str("a = nan").unwrap();
        assert!(t["a"].as_float().unwrap().is_nan());
    }

    #[test]
    fn bool_true() {
        let t = shiguredo_toml::from_str("a = true").unwrap();
        assert!(t["a"].as_bool().unwrap());
    }

    #[test]
    fn bool_false() {
        let t = shiguredo_toml::from_str("a = false").unwrap();
        assert!(!t["a"].as_bool().unwrap());
    }

    #[test]
    fn array_basic() {
        let t = shiguredo_toml::from_str("a = [1, 2, 3]").unwrap();
        let arr = t["a"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_integer().unwrap(), 1);
    }

    #[test]
    fn array_trailing_comma() {
        let t = shiguredo_toml::from_str("a = [1, 2, 3,]").unwrap();
        let arr = t["a"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn array_multiline() {
        let t = shiguredo_toml::from_str("a = [\n1,\n2,\n3\n]").unwrap();
        let arr = t["a"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn inline_table() {
        let t = shiguredo_toml::from_str("a = {b = 1, c = \"hello\"}").unwrap();
        let inner = t["a"].as_table().unwrap();
        assert_eq!(inner["b"].as_integer().unwrap(), 1);
        assert_eq!(inner["c"].as_str().unwrap(), "hello");
    }

    #[test]
    fn table_header() {
        let t = shiguredo_toml::from_str("[server]\nhost = \"localhost\"\nport = 8080").unwrap();
        let server = t["server"].as_table().unwrap();
        assert_eq!(server["host"].as_str().unwrap(), "localhost");
        assert_eq!(server["port"].as_integer().unwrap(), 8080);
    }

    #[test]
    fn nested_table() {
        let t = shiguredo_toml::from_str("[a.b]\nc = 1").unwrap();
        let c = t["a"].as_table().unwrap()["b"].as_table().unwrap()["c"]
            .as_integer()
            .unwrap();
        assert_eq!(c, 1);
    }

    #[test]
    fn array_of_tables() {
        let t = shiguredo_toml::from_str("[[item]]\nname = \"a\"\n[[item]]\nname = \"b\"").unwrap();
        let items = t["item"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_table().unwrap()["name"].as_str().unwrap(), "a");
        assert_eq!(items[1].as_table().unwrap()["name"].as_str().unwrap(), "b");
    }

    #[test]
    fn dotted_key() {
        let t = shiguredo_toml::from_str("a.b.c = 1").unwrap();
        let c = t["a"].as_table().unwrap()["b"].as_table().unwrap()["c"]
            .as_integer()
            .unwrap();
        assert_eq!(c, 1);
    }

    #[test]
    fn quoted_key() {
        let t = shiguredo_toml::from_str("\"hello world\" = 1").unwrap();
        assert_eq!(t["hello world"].as_integer().unwrap(), 1);
    }

    #[test]
    fn empty_quoted_key() {
        let t = shiguredo_toml::from_str("\"\" = 1").unwrap();
        assert_eq!(t[""].as_integer().unwrap(), 1);
    }

    #[test]
    fn comment() {
        let t = shiguredo_toml::from_str("# comment\na = 1 # inline").unwrap();
        assert_eq!(t["a"].as_integer().unwrap(), 1);
    }

    #[test]
    fn unicode_escape() {
        let t = shiguredo_toml::from_str(r#"a = "\u3042""#).unwrap();
        assert_eq!(t["a"].as_str().unwrap(), "\u{3042}");
    }

    #[test]
    fn unicode_escape_8digit() {
        let t = shiguredo_toml::from_str(r#"a = "\U0001F600""#).unwrap();
        assert_eq!(t["a"].as_str().unwrap(), "\u{1F600}");
    }

    #[test]
    fn datetime_in_document() {
        let t = shiguredo_toml::from_str("a = 1979-05-27T07:32:00Z").unwrap();
        let dt = t["a"].as_datetime().unwrap();
        assert_eq!(dt.date.as_ref().unwrap().year, 1979);
    }

    #[test]
    fn multiline_basic_string_line_ending_backslash() {
        let t = shiguredo_toml::from_str("a = \"\"\"\nhello \\\n  \n  world\"\"\"").unwrap();
        assert_eq!(t["a"].as_str().unwrap(), "hello world");
    }

    #[test]
    fn mixed_types_array() {
        // TOML v1.0.0 は異種配列を許容する
        let t = shiguredo_toml::from_str("a = [1, \"two\", 3.0]").unwrap();
        let arr = t["a"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert!(arr[0].is_integer());
        assert!(arr[1].is_str());
        assert!(arr[2].is_float());
    }

    #[test]
    fn implicit_table_via_dotted_key() {
        let t = shiguredo_toml::from_str("a.b = 1\na.c = 2").unwrap();
        let a = t["a"].as_table().unwrap();
        assert_eq!(a["b"].as_integer().unwrap(), 1);
        assert_eq!(a["c"].as_integer().unwrap(), 2);
    }

    #[test]
    fn super_table_implicit() {
        let input = "[a.b]\nc = 1\n[a]\nd = 2";
        let t = shiguredo_toml::from_str(input).unwrap();
        let a = t["a"].as_table().unwrap();
        assert_eq!(a["d"].as_integer().unwrap(), 2);
        assert_eq!(a["b"].as_table().unwrap()["c"].as_integer().unwrap(), 1);
    }

    #[test]
    fn integer_zero() {
        let t = shiguredo_toml::from_str("a = 0").unwrap();
        assert_eq!(t["a"].as_integer().unwrap(), 0);
    }

    #[test]
    fn float_zero() {
        let t = shiguredo_toml::from_str("a = 0.0").unwrap();
        assert_eq!(t["a"].as_float().unwrap(), 0.0);
    }

    #[test]
    fn multiline_basic_extra_quotes() {
        // 4 or 5 quotes at end: """..."""" or """..."""""
        let t = shiguredo_toml::from_str("a = \"\"\"hello\"\"\"\"\"").unwrap();
        assert_eq!(t["a"].as_str().unwrap(), "hello\"\"");
    }

    #[test]
    fn multiline_literal_extra_quotes() {
        let t = shiguredo_toml::from_str("a = '''hello'''''").unwrap();
        assert_eq!(t["a"].as_str().unwrap(), "hello''");
    }

    #[test]
    fn value_type_name() {
        assert_eq!(Value::String("".into()).type_name(), "string");
        assert_eq!(Value::Integer(0).type_name(), "integer");
        assert_eq!(Value::Float(0.0).type_name(), "float");
        assert_eq!(Value::Boolean(true).type_name(), "boolean");
        assert_eq!(Value::Array(vec![]).type_name(), "array");
    }
}
