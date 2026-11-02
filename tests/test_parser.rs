mod parse_error {
    #[test]
    fn empty_input() {
        let result = shiguredo_toml::from_str("");
        assert!(result.is_ok());
        assert!(result.expect("結果は Ok になるはず").is_empty());
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
        let t = shiguredo_toml::from_str(r#"a = "hello""#).expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_str().expect("値は文字列になるはず"), "hello");
    }

    #[test]
    fn basic_string_escapes() {
        let t =
            shiguredo_toml::from_str(r#"a = "hello\nworld""#).expect("TOML のパースに成功するはず");
        assert_eq!(
            t["a"].as_str().expect("値は文字列になるはず"),
            "hello\nworld"
        );
    }

    #[test]
    fn literal_string() {
        let t = shiguredo_toml::from_str(r#"a = 'C:\Users\path'"#)
            .expect("TOML のパースに成功するはず");
        assert_eq!(
            t["a"].as_str().expect("値は文字列になるはず"),
            "C:\\Users\\path"
        );
    }

    #[test]
    fn multiline_basic_string() {
        let t = shiguredo_toml::from_str("a = \"\"\"\nhello\nworld\"\"\"")
            .expect("TOML のパースに成功するはず");
        assert_eq!(
            t["a"].as_str().expect("値は文字列になるはず"),
            "hello\nworld"
        );
    }

    #[test]
    fn multiline_literal_string() {
        let t = shiguredo_toml::from_str("a = '''\nhello\nworld'''")
            .expect("TOML のパースに成功するはず");
        assert_eq!(
            t["a"].as_str().expect("値は文字列になるはず"),
            "hello\nworld"
        );
    }

    #[test]
    fn integer_decimal() {
        let t = shiguredo_toml::from_str("a = 42").expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), 42);
    }

    #[test]
    fn integer_negative() {
        let t = shiguredo_toml::from_str("a = -42").expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), -42);
    }

    #[test]
    fn integer_positive_sign() {
        let t = shiguredo_toml::from_str("a = +42").expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), 42);
    }

    #[test]
    fn integer_hex() {
        let t = shiguredo_toml::from_str("a = 0xDEAD_BEEF").expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), 0xDEADBEEF);
    }

    #[test]
    fn integer_oct() {
        let t = shiguredo_toml::from_str("a = 0o755").expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), 0o755);
    }

    #[test]
    fn integer_bin() {
        let t = shiguredo_toml::from_str("a = 0b1101").expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), 0b1101);
    }

    #[test]
    fn integer_underscore() {
        let t = shiguredo_toml::from_str("a = 1_000_000").expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), 1_000_000);
    }

    #[test]
    fn float_basic() {
        let t = shiguredo_toml::from_str("a = 2.72").expect("TOML のパースに成功するはず");
        let f = t["a"].as_float().expect("値は浮動小数点数になるはず");
        assert!((f - 2.72).abs() < 1e-10);
    }

    #[test]
    fn float_exponent() {
        let t = shiguredo_toml::from_str("a = 5e+22").expect("TOML のパースに成功するはず");
        let f = t["a"].as_float().expect("値は浮動小数点数になるはず");
        assert!((f - 5e22).abs() < 1e12);
    }

    #[test]
    fn float_inf() {
        let t = shiguredo_toml::from_str("a = inf").expect("TOML のパースに成功するはず");
        assert!(
            t["a"]
                .as_float()
                .expect("値は浮動小数点数になるはず")
                .is_infinite()
        );
        assert!(
            t["a"]
                .as_float()
                .expect("値は浮動小数点数になるはず")
                .is_sign_positive()
        );
    }

    #[test]
    fn float_neg_inf() {
        let t = shiguredo_toml::from_str("a = -inf").expect("TOML のパースに成功するはず");
        assert!(
            t["a"]
                .as_float()
                .expect("値は浮動小数点数になるはず")
                .is_infinite()
        );
        assert!(
            t["a"]
                .as_float()
                .expect("値は浮動小数点数になるはず")
                .is_sign_negative()
        );
    }

    #[test]
    fn float_nan() {
        let t = shiguredo_toml::from_str("a = nan").expect("TOML のパースに成功するはず");
        assert!(
            t["a"]
                .as_float()
                .expect("値は浮動小数点数になるはず")
                .is_nan()
        );
    }

    #[test]
    fn bool_true() {
        let t = shiguredo_toml::from_str("a = true").expect("TOML のパースに成功するはず");
        assert!(t["a"].as_bool().expect("値はブール値になるはず"));
    }

    #[test]
    fn bool_false() {
        let t = shiguredo_toml::from_str("a = false").expect("TOML のパースに成功するはず");
        assert!(!t["a"].as_bool().expect("値はブール値になるはず"));
    }

    #[test]
    fn array_basic() {
        let t = shiguredo_toml::from_str("a = [1, 2, 3]").expect("TOML のパースに成功するはず");
        let arr = t["a"].as_array().expect("値は配列になるはず");
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_integer().expect("値は整数になるはず"), 1);
    }

    #[test]
    fn array_trailing_comma() {
        let t = shiguredo_toml::from_str("a = [1, 2, 3,]").expect("TOML のパースに成功するはず");
        let arr = t["a"].as_array().expect("値は配列になるはず");
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn array_multiline() {
        let t =
            shiguredo_toml::from_str("a = [\n1,\n2,\n3\n]").expect("TOML のパースに成功するはず");
        let arr = t["a"].as_array().expect("値は配列になるはず");
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn inline_table() {
        let t = shiguredo_toml::from_str("a = {b = 1, c = \"hello\"}")
            .expect("TOML のパースに成功するはず");
        let inner = t["a"].as_table().expect("値はテーブルになるはず");
        assert_eq!(inner["b"].as_integer().expect("値は整数になるはず"), 1);
        assert_eq!(inner["c"].as_str().expect("値は文字列になるはず"), "hello");
    }

    #[test]
    fn table_header() {
        let t = shiguredo_toml::from_str("[server]\nhost = \"localhost\"\nport = 8080")
            .expect("TOML のパースに成功するはず");
        let server = t["server"].as_table().expect("値はテーブルになるはず");
        assert_eq!(
            server["host"].as_str().expect("値は文字列になるはず"),
            "localhost"
        );
        assert_eq!(
            server["port"].as_integer().expect("値は整数になるはず"),
            8080
        );
    }

    #[test]
    fn nested_table() {
        let t = shiguredo_toml::from_str("[a.b]\nc = 1").expect("TOML のパースに成功するはず");
        let c = t["a"].as_table().expect("値はテーブルになるはず")["b"]
            .as_table()
            .expect("値はテーブルになるはず")["c"]
            .as_integer()
            .expect("値は整数になるはず");
        assert_eq!(c, 1);
    }

    #[test]
    fn array_of_tables() {
        let t = shiguredo_toml::from_str("[[item]]\nname = \"a\"\n[[item]]\nname = \"b\"")
            .expect("TOML のパースに成功するはず");
        let items = t["item"].as_array().expect("値は配列になるはず");
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].as_table().expect("値はテーブルになるはず")["name"]
                .as_str()
                .expect("値は文字列になるはず"),
            "a"
        );
        assert_eq!(
            items[1].as_table().expect("値はテーブルになるはず")["name"]
                .as_str()
                .expect("値は文字列になるはず"),
            "b"
        );
    }

    #[test]
    fn dotted_key() {
        let t = shiguredo_toml::from_str("a.b.c = 1").expect("TOML のパースに成功するはず");
        let c = t["a"].as_table().expect("値はテーブルになるはず")["b"]
            .as_table()
            .expect("値はテーブルになるはず")["c"]
            .as_integer()
            .expect("値は整数になるはず");
        assert_eq!(c, 1);
    }

    #[test]
    fn quoted_key() {
        let t =
            shiguredo_toml::from_str("\"hello world\" = 1").expect("TOML のパースに成功するはず");
        assert_eq!(
            t["hello world"].as_integer().expect("値は整数になるはず"),
            1
        );
    }

    #[test]
    fn empty_quoted_key() {
        let t = shiguredo_toml::from_str("\"\" = 1").expect("TOML のパースに成功するはず");
        assert_eq!(t[""].as_integer().expect("値は整数になるはず"), 1);
    }

    #[test]
    fn comment() {
        let t = shiguredo_toml::from_str("# comment\na = 1 # inline")
            .expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), 1);
    }

    #[test]
    fn unicode_escape() {
        let t = shiguredo_toml::from_str(r#"a = "\u3042""#).expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_str().expect("値は文字列になるはず"), "\u{3042}");
    }

    #[test]
    fn unicode_escape_8digit() {
        let t =
            shiguredo_toml::from_str(r#"a = "\U0001F600""#).expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_str().expect("値は文字列になるはず"), "\u{1F600}");
    }

    #[test]
    fn datetime_in_document() {
        let t = shiguredo_toml::from_str("a = 1979-05-27T07:32:00Z")
            .expect("TOML のパースに成功するはず");
        let dt = t["a"].as_datetime().expect("値は日時になるはず");
        assert_eq!(
            dt.date.as_ref().expect("フィールドは設定されるはず").year,
            1979
        );
    }

    #[test]
    fn multiline_basic_string_line_ending_backslash() {
        let t = shiguredo_toml::from_str("a = \"\"\"\nhello \\\n  \n  world\"\"\"")
            .expect("TOML のパースに成功するはず");
        assert_eq!(
            t["a"].as_str().expect("値は文字列になるはず"),
            "hello world"
        );
    }

    #[test]
    fn mixed_types_array() {
        // TOML v1.0.0 は異種配列を許容する
        let t =
            shiguredo_toml::from_str("a = [1, \"two\", 3.0]").expect("TOML のパースに成功するはず");
        let arr = t["a"].as_array().expect("値は配列になるはず");
        assert_eq!(arr.len(), 3);
        assert!(arr[0].is_integer());
        assert!(arr[1].is_str());
        assert!(arr[2].is_float());
    }

    #[test]
    fn implicit_table_via_dotted_key() {
        let t = shiguredo_toml::from_str("a.b = 1\na.c = 2").expect("TOML のパースに成功するはず");
        let a = t["a"].as_table().expect("値はテーブルになるはず");
        assert_eq!(a["b"].as_integer().expect("値は整数になるはず"), 1);
        assert_eq!(a["c"].as_integer().expect("値は整数になるはず"), 2);
    }

    #[test]
    fn super_table_implicit() {
        let input = "[a.b]\nc = 1\n[a]\nd = 2";
        let t = shiguredo_toml::from_str(input).expect("TOML のパースに成功するはず");
        let a = t["a"].as_table().expect("値はテーブルになるはず");
        assert_eq!(a["d"].as_integer().expect("値は整数になるはず"), 2);
        assert_eq!(
            a["b"].as_table().expect("値はテーブルになるはず")["c"]
                .as_integer()
                .expect("値は整数になるはず"),
            1
        );
    }

    #[test]
    fn integer_zero() {
        let t = shiguredo_toml::from_str("a = 0").expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), 0);
    }

    #[test]
    fn float_zero() {
        let t = shiguredo_toml::from_str("a = 0.0").expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_float().expect("値は浮動小数点数になるはず"), 0.0);
    }

    #[test]
    fn multiline_basic_extra_quotes() {
        // 4 or 5 quotes at end: """..."""" or """..."""""
        let t = shiguredo_toml::from_str("a = \"\"\"hello\"\"\"\"\"")
            .expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_str().expect("値は文字列になるはず"), "hello\"\"");
    }

    #[test]
    fn multiline_literal_extra_quotes() {
        let t = shiguredo_toml::from_str("a = '''hello'''''").expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_str().expect("値は文字列になるはず"), "hello''");
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

mod v1_1 {
    use shiguredo_toml::{TomlVersion, from_str_with_version};

    /// \e エスケープが U+001B (ESC) になることを確認する。
    #[test]
    fn escape_e_v1_1() {
        let t = from_str_with_version("a = \"\\e\"", TomlVersion::V1_1)
            .expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_str().expect("値は文字列になるはず"), "\u{001B}");
    }

    /// \x1B エスケープが U+001B (ESC) になることを確認する。
    #[test]
    fn escape_x_v1_1() {
        let t = from_str_with_version("a = \"\\x1B\"", TomlVersion::V1_1)
            .expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_str().expect("値は文字列になるはず"), "\u{001B}");
    }

    /// \x00 エスケープが U+0000 (NUL) になることを確認する。
    #[test]
    fn escape_x_nul_v1_1() {
        let t = from_str_with_version("a = \"\\x00\"", TomlVersion::V1_1)
            .expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_str().expect("値は文字列になるはず"), "\u{0000}");
    }

    /// V1_0 で \e がエラーになることを確認する。
    #[test]
    fn escape_e_v1_0_errors() {
        let result = from_str_with_version("a = \"\\e\"", TomlVersion::V1_0);
        assert!(result.is_err());
    }

    /// V1_0 で \x がエラーになることを確認する。
    #[test]
    fn escape_x_v1_0_errors() {
        let result = from_str_with_version("a = \"\\x1B\"", TomlVersion::V1_0);
        assert!(result.is_err());
    }

    /// V1_1 でインラインテーブルの末尾カンマが許可されることを確認する。
    #[test]
    fn inline_table_trailing_comma_v1_1() {
        let t = from_str_with_version("a = {b = 1,}", TomlVersion::V1_1)
            .expect("TOML のパースに成功するはず");
        assert_eq!(t["a"]["b"].as_integer().expect("値は整数になるはず"), 1);
    }

    /// V1_1 でインラインテーブルの複数行が許可されることを確認する。
    #[test]
    fn inline_table_multiline_v1_1() {
        let input = "a = {\n  b = 1,\n  c = 2,\n}";
        let t =
            from_str_with_version(input, TomlVersion::V1_1).expect("TOML のパースに成功するはず");
        assert_eq!(t["a"]["b"].as_integer().expect("値は整数になるはず"), 1);
        assert_eq!(t["a"]["c"].as_integer().expect("値は整数になるはず"), 2);
    }

    /// V1_0 でインラインテーブルの末尾カンマがエラーのままであることを確認する。
    #[test]
    fn inline_table_trailing_comma_v1_0_errors() {
        let result = from_str_with_version("a = {b = 1,}", TomlVersion::V1_0);
        assert!(result.is_err());
    }

    /// V1_1 でインラインテーブル内の改行がエラーのままであることを V1_0 で確認する。
    #[test]
    fn inline_table_multiline_v1_0_errors() {
        let input = "a = {\n  b = 1\n}";
        let result = from_str_with_version(input, TomlVersion::V1_0);
        assert!(result.is_err());
    }

    /// V1_1 で秒省略の時刻が 07:32:00 として解析されることを確認する。
    #[test]
    fn datetime_without_seconds_v1_1() {
        let t = from_str_with_version("t = 07:32", TomlVersion::V1_1)
            .expect("TOML のパースに成功するはず");
        let dt = t["t"].as_datetime().expect("値は日時になるはず");
        let time = dt.time.as_ref().expect("フィールドは設定されるはず");
        assert_eq!(time.hour, 7);
        assert_eq!(time.minute, 32);
        assert_eq!(time.second, 0);
    }

    /// V1_0 で秒省略の時刻がエラーになることを確認する。
    #[test]
    fn datetime_without_seconds_v1_0_errors() {
        let result = from_str_with_version("t = 07:32", TomlVersion::V1_0);
        assert!(result.is_err());
    }

    /// V1_1 で複数行リテラル文字列の CRLF が LF に正規化されることを確認する。
    #[test]
    fn ml_literal_string_crlf_normalized_v1_1() {
        let input = "a = '''\r\nhello\r\nworld\r\n'''";
        let t =
            from_str_with_version(input, TomlVersion::V1_1).expect("TOML のパースに成功するはず");
        // 開始直後の CRLF は削除され、内部の CRLF は LF に正規化される
        assert_eq!(
            t["a"].as_str().expect("値は文字列になるはず"),
            "hello\nworld\n"
        );
    }

    /// V1_0 で複数行リテラル文字列の CR が保持されることを確認する。
    #[test]
    fn ml_literal_string_cr_preserved_v1_0() {
        let input = "a = '''\nhello\r\nworld\n'''";
        let t =
            from_str_with_version(input, TomlVersion::V1_0).expect("TOML のパースに成功するはず");
        // V1_0 では CRLF がそのまま保持される
        assert_eq!(
            t["a"].as_str().expect("値は文字列になるはず"),
            "hello\r\nworld\n"
        );
    }
}

mod array_table_state_reset {
    use shiguredo_toml::{TomlVersion, from_str_with_version};

    /// 再現入力: 前要素で定義した配列テーブルのサブパスが残存しても、
    /// 新要素配下の標準テーブル定義が成功する。
    ///
    /// 新要素配下のヘッダ `[a.b.c.d]` が残存パス `a.b.c` を途中パスとして
    /// 経由する点が特徴で、修正前は `internal error: array table 'c' not found`
    /// で拒否されていた。
    ///
    /// TOML v1.0.0 / v1.1.0 仕様の「Array of Tables」節: 配列テーブルへの参照は
    /// 常に最後に定義された要素を指すため、再オープン後の新要素配下に
    /// サブテーブルを定義できる。
    #[test]
    fn subtable_under_new_element_after_reopen() {
        let input = "[[a]]\n[a.b]\n[[a.b.c]]\n[a.b.c.d]\nw = 1\n[[a]]\n[a.b.c.d]\n";
        for version in [TomlVersion::V1_0, TomlVersion::V1_1] {
            let Ok(t) = from_str_with_version(input, version) else {
                panic!("TOML のパースに成功するはず（{version:?}）");
            };
            let a = t["a"].as_array().expect("値は配列になるはず");
            assert_eq!(a.len(), 2);
            // 1 番目の要素の b.c は配列テーブルで、その要素配下に d = { w = 1 } が定義される
            let first = a[0].as_table().expect("値はテーブルになるはず");
            let first_c = first["b"]["c"].as_array().expect("値は配列になるはず");
            assert_eq!(first_c.len(), 1);
            assert_eq!(
                first_c[0].as_table().expect("値はテーブルになるはず")["d"]["w"]
                    .as_integer()
                    .expect("値は整数になるはず"),
                1
            );
            // 2 番目の要素配下の b.c と b.c.d が標準テーブルとして定義される
            let second = a[1].as_table().expect("値はテーブルになるはず");
            assert!(
                second["b"]["c"].as_table().is_some(),
                "b.c は標準テーブルになるはず"
            );
            assert!(
                second["b"]["c"].as_array().is_none(),
                "b.c は配列にならないはず"
            );
            assert!(
                second["b"]["c"]["d"].as_table().is_some(),
                "b.c.d は標準テーブルになるはず"
            );
            // 新要素には前要素の w = 1 が引き継がれない
            assert!(
                second["b"]["c"]["d"].get("w").is_none(),
                "新要素は前要素の値を引き継がないはず"
            );
        }
    }

    /// 同根ケース: 前要素で配列テーブルだった名前を、新要素配下で
    /// 標準テーブルとして再定義できる。
    ///
    /// 再現入力との違いは、新要素配下のヘッダが `[a.b.c]`（残存パスが
    /// 終端に来る）である点。修正前は残存パスにより
    /// `'a.b.c' is defined as an array table and cannot be a standard table`
    /// で拒否されていた。
    ///
    /// TOML v1.0.0 / v1.1.0 仕様の「Array of Tables」節: 配列テーブルへの参照は
    /// 常に最後に定義された要素を指す。新要素配下では前要素のサブパスは
    /// 未定義扱いになるため、前要素で配列テーブルだった名前を
    /// 標準テーブルとして定義できる。
    #[test]
    fn standard_table_replacing_previous_array_table() {
        let input = "[[a]]\n[a.b]\n[[a.b.c]]\n[a.b.c.d]\nw = 1\n[[a]]\n[a.b.c]\n";
        for version in [TomlVersion::V1_0, TomlVersion::V1_1] {
            let Ok(t) = from_str_with_version(input, version) else {
                panic!("TOML のパースに成功するはず（{version:?}）");
            };
            let a = t["a"].as_array().expect("値は配列になるはず");
            assert_eq!(a.len(), 2);
            // 1 番目の要素の b.c は配列テーブルのまま w = 1 を保持する
            let first = a[0].as_table().expect("値はテーブルになるはず");
            let first_c = first["b"]["c"].as_array().expect("値は配列になるはず");
            assert_eq!(
                first_c[0].as_table().expect("値はテーブルになるはず")["d"]["w"]
                    .as_integer()
                    .expect("値は整数になるはず"),
                1
            );
            // 2 番目の要素配下の b.c が標準テーブル（配列ではない）になる
            let second = a[1].as_table().expect("値はテーブルになるはず");
            assert!(
                second["b"]["c"].as_table().is_some(),
                "b.c は標準テーブルになるはず"
            );
            assert!(
                second["b"]["c"].as_array().is_none(),
                "b.c は配列にならないはず"
            );
            // 新要素には前要素の d.w = 1 が引き継がれない
            assert!(
                second["b"]["c"].get("d").is_none(),
                "新要素は前要素の値を引き継がないはず"
            );
        }
    }

    /// 前要素で配列テーブルが定義されていた途中パスを、新要素配下で
    /// 標準テーブルとして作成し直した上で配列テーブルを新規定義できる。
    ///
    /// TOML v1.0.0 / v1.1.0 仕様の「Array of Tables」節: 配列テーブルへの参照は
    /// 常に最後に定義された要素を指す。新要素配下では前要素のサブパスは
    /// 未定義扱いになるため、途中パスを標準テーブルとして作成し直した上で
    /// 配列テーブルを新規定義できる。
    #[test]
    fn new_array_table_under_reopened_element() {
        let input = "[[a]]\n[[a.b]]\nw = 1\n[[a]]\n[[a.b.c]]\nz = 1\n";
        for version in [TomlVersion::V1_0, TomlVersion::V1_1] {
            let Ok(t) = from_str_with_version(input, version) else {
                panic!("TOML のパースに成功するはず（{version:?}）");
            };
            let a = t["a"].as_array().expect("値は配列になるはず");
            assert_eq!(a.len(), 2);
            // 1 番目の要素の b は配列テーブルのまま w = 1 を保持する
            let first = a[0].as_table().expect("値はテーブルになるはず");
            let first_b = first["b"].as_array().expect("値は配列になるはず");
            assert_eq!(
                first_b[0].as_table().expect("値はテーブルになるはず")["w"]
                    .as_integer()
                    .expect("値は整数になるはず"),
                1
            );
            let second = a[1].as_table().expect("値はテーブルになるはず");
            // 2 番目の要素配下の b は標準テーブルとして新規作成される
            assert!(
                second["b"].as_table().is_some(),
                "b は標準テーブルになるはず"
            );
            assert!(second["b"].as_array().is_none(), "b は配列にならないはず");
            // b.c は配列テーブルとして新規作成される
            let bc = second["b"]["c"].as_array().expect("値は配列になるはず");
            assert_eq!(bc.len(), 1);
            assert_eq!(
                bc[0].as_table().expect("値はテーブルになるはず")["z"]
                    .as_integer()
                    .expect("値は整数になるはず"),
                1
            );
            // 新要素には前要素の w = 1 が引き継がれない
            assert!(
                second["b"].get("w").is_none(),
                "新要素は前要素の値を引き継がないはず"
            );
        }
    }

    /// 回帰テスト: 新要素配下で前要素と同じ配列テーブル名を再定義すると、
    /// 前要素とは別の新規配列が作成される（仕様準拠の既存挙動）。
    ///
    /// TOML v1.0.0 / v1.1.0 仕様の「Array of Tables」節: ヘッダの再出現は
    /// 新要素を生成し、参照は常に最後に定義された要素を指す。新要素配下では
    /// 前要素と同じ配列テーブル名は未定義扱いになるため、別の新規配列として
    /// 作成される。
    #[test]
    fn same_array_table_name_under_new_element() {
        let input = "[[a]]\n[[a.b]]\nw = 1\n[[a]]\n[[a.b]]\nx = 1\n";
        for version in [TomlVersion::V1_0, TomlVersion::V1_1] {
            let Ok(t) = from_str_with_version(input, version) else {
                panic!("TOML のパースに成功するはず（{version:?}）");
            };
            let a = t["a"].as_array().expect("値は配列になるはず");
            assert_eq!(a.len(), 2);
            // 1 番目の要素の b は w = 1 を持つ配列
            let first_b = a[0].as_table().expect("値はテーブルになるはず")["b"]
                .as_array()
                .expect("値は配列になるはず");
            assert_eq!(first_b.len(), 1);
            let first_b_element = first_b[0].as_table().expect("値はテーブルになるはず");
            assert_eq!(
                first_b_element["w"]
                    .as_integer()
                    .expect("値は整数になるはず"),
                1
            );
            // 1 番目の要素には 2 番目の要素の x = 1 が混入しない
            assert!(
                first_b_element.get("x").is_none(),
                "1 番目の要素は 2 番目の要素の値を引き継がないはず"
            );
            // 2 番目の要素の b は前要素とは別の新規配列で、x = 1 を持つ
            let second_b = a[1].as_table().expect("値はテーブルになるはず")["b"]
                .as_array()
                .expect("値は配列になるはず");
            assert_eq!(second_b.len(), 1);
            let second_b_element = second_b[0].as_table().expect("値はテーブルになるはず");
            assert_eq!(
                second_b_element["x"]
                    .as_integer()
                    .expect("値は整数になるはず"),
                1
            );
            assert!(
                second_b_element.get("w").is_none(),
                "新要素は前要素の値を引き継がないはず"
            );
        }
    }

    /// 前要素で配列テーブルが定義されていた途中パスを、新要素配下で
    /// 標準テーブルの親パスとして使える。
    ///
    /// TOML v1.0.0 / v1.1.0 仕様の「Array of Tables」節: 配列テーブルへの参照は
    /// 常に最後に定義された要素を指す。新要素配下では前要素のサブパスは
    /// 未定義扱いになるため、標準テーブルの親パスとして使える。
    #[test]
    fn standard_table_using_previous_array_table_as_parent() {
        let input = "[[a]]\n[[a.b]]\nw = 1\n[[a]]\n[a.b.c]\n";
        for version in [TomlVersion::V1_0, TomlVersion::V1_1] {
            let Ok(t) = from_str_with_version(input, version) else {
                panic!("TOML のパースに成功するはず（{version:?}）");
            };
            let a = t["a"].as_array().expect("値は配列になるはず");
            assert_eq!(a.len(), 2);
            // 1 番目の要素の b は配列テーブルのまま w = 1 を保持する
            let first_b = a[0].as_table().expect("値はテーブルになるはず")["b"]
                .as_array()
                .expect("値は配列になるはず");
            assert_eq!(first_b.len(), 1);
            assert_eq!(
                first_b[0].as_table().expect("値はテーブルになるはず")["w"]
                    .as_integer()
                    .expect("値は整数になるはず"),
                1
            );
            // 1 番目の要素には新要素の定義が混入しない
            assert!(
                first_b[0]
                    .as_table()
                    .expect("値はテーブルになるはず")
                    .get("c")
                    .is_none(),
                "1 番目の要素は 2 番目の要素の値を引き継がないはず"
            );
            // 2 番目の要素配下の b.c が標準テーブルとして定義される
            let second = a[1].as_table().expect("値はテーブルになるはず");
            assert!(
                second["b"]["c"].as_table().is_some(),
                "b.c は標準テーブルになるはず"
            );
            assert!(
                second["b"]["c"].as_array().is_none(),
                "b.c は配列にならないはず"
            );
        }
    }

    /// 中段の配列テーブル（`[[a.b]]`）を再オープンした場合も、新要素配下の
    /// サブテーブル定義が成功する。
    ///
    /// TOML v1.0.0 / v1.1.0 仕様の「Array of Tables」節: 配列テーブルへの参照は
    /// 常に最後に定義された要素を指す。ネストの深さに関わらず同じ規則が
    /// 適用される。
    #[test]
    fn nested_array_table_reopen() {
        let input = "[[a]]\n[[a.b]]\n[[a.b.c]]\n[[a.b]]\n[a.b.c]\n";
        for version in [TomlVersion::V1_0, TomlVersion::V1_1] {
            let Ok(t) = from_str_with_version(input, version) else {
                panic!("TOML のパースに成功するはず（{version:?}）");
            };
            let a = t["a"].as_array().expect("値は配列になるはず");
            assert_eq!(a.len(), 1);
            let b = a[0].as_table().expect("値はテーブルになるはず")["b"]
                .as_array()
                .expect("値は配列になるはず");
            assert_eq!(b.len(), 2);
            // 1 番目の要素の b.c は配列テーブルのまま（要素数 1）
            let first_b = b[0].as_table().expect("値はテーブルになるはず");
            let first_c = first_b["c"].as_array().expect("値は配列になるはず");
            assert_eq!(first_c.len(), 1);
            // 2 番目の要素配下の b.c が標準テーブルとして定義される
            let second_b = b[1].as_table().expect("値はテーブルになるはず");
            assert!(
                second_b["c"].as_table().is_some(),
                "b.c は標準テーブルになるはず"
            );
            assert!(
                second_b["c"].as_array().is_none(),
                "b.c は配列にならないはず"
            );
        }
    }
}

mod bom {
    /// 先頭 BOM の直後に改行がある場合にパース成功する。
    /// toml-test の valid/utf8-bom-01 相当。
    #[test]
    fn leading_bom_then_newline_and_comment() {
        let input = "\u{FEFF}# comment\na = 1\n";
        let t = shiguredo_toml::from_str(input).expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), 1);
    }

    /// 先頭 BOM の直後にキー=値が続く場合にパース成功する。
    /// toml-test の valid/utf8-bom-02 相当。
    #[test]
    fn leading_bom_then_keyval_and_comment() {
        let input = "\u{FEFF}a=1# comment\n";
        let t = shiguredo_toml::from_str(input).expect("TOML のパースに成功するはず");
        assert_eq!(t["a"].as_integer().expect("値は整数になるはず"), 1);
    }

    /// 先頭 BOM のみで内容が空の場合にパース成功する。
    #[test]
    fn only_bom() {
        let t = shiguredo_toml::from_str("\u{FEFF}").expect("TOML のパースに成功するはず");
        assert!(t.is_empty());
    }

    /// 先頭 BOM が連続する場合は 2 個目以降が無効文字としてエラーになる。
    #[test]
    fn double_bom_is_error() {
        let result = shiguredo_toml::from_str("\u{FEFF}\u{FEFF}a = 1");
        assert!(result.is_err());
    }

    /// 中間 (キーの前など) に BOM が出現する場合はエラーになる。
    #[test]
    fn bom_in_middle_is_error() {
        let result = shiguredo_toml::from_str("a = 1\n\u{FEFF}b = 2");
        assert!(result.is_err());
    }

    /// 値の中に BOM が出現する場合はエラーになる（ベア値として解釈不可）。
    #[test]
    fn bom_in_value_is_error() {
        let result = shiguredo_toml::from_str("a = \u{FEFF}1");
        assert!(result.is_err());
    }
}
