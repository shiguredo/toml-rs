mod basic_types {
    #[test]
    fn string_integer_float_bool() {
        let table = shiguredo_toml::from_str(
            r#"
str = "hello"
int = 42
float = 3.14
bool_true = true
bool_false = false
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn special_floats() {
        let table = shiguredo_toml::from_str(
            r#"
pos_inf = inf
neg_inf = -inf
nan_val = nan
"#,
        )
        .expect("TOML のパースに成功するはず");
        // NaN は Debug 出力で NaN になるのでスナップショット可能
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn integer_formats() {
        let table = shiguredo_toml::from_str(
            r#"
dec = 1_000
hex = 0xDEAD
oct = 0o755
bin = 0b1101
positive = +99
negative = -17
zero = 0
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn string_variants() {
        let table = shiguredo_toml::from_str(
            r#"
basic = "hello\nworld"
literal = 'C:\Users\path'
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn multiline_strings() {
        let input = "ml_basic = \"\"\"\nhello\nworld\"\"\"\nml_literal = '''\nhello\nworld'''";
        let table = shiguredo_toml::from_str(input).expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn datetime_variants() {
        let table = shiguredo_toml::from_str(
            r#"
odt = 1979-05-27T07:32:00Z
odt_offset = 1979-05-27T07:32:00+09:00
ldt = 1979-05-27T07:32:00
ld = 1979-05-27
lt = 07:32:00
lt_frac = 07:32:00.123456789
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }
}

mod structure {
    #[test]
    fn nested_tables() {
        let table = shiguredo_toml::from_str(
            r#"
[server]
host = "localhost"
port = 8080

[server.tls]
enabled = true
cert = "/path/to/cert.pem"

[database]
name = "mydb"
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn array_of_tables() {
        let table = shiguredo_toml::from_str(
            r#"
[[products]]
name = "Hammer"
sku = 738594937

[[products]]
name = "Nail"
sku = 284758393
color = "gray"
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn nested_array_of_tables() {
        // TOML v1.0.0 仕様のサンプル
        let table = shiguredo_toml::from_str(
            r#"
[[fruits]]
name = "apple"

[[fruits.varieties]]
name = "red delicious"

[[fruits.varieties]]
name = "granny smith"

[[fruits]]
name = "banana"

[[fruits.varieties]]
name = "plantain"
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn inline_table() {
        let table = shiguredo_toml::from_str(
            r#"
point = {x = 1, y = 2}
animal = {type.name = "pug"}
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn arrays() {
        let table = shiguredo_toml::from_str(
            r#"
integers = [1, 2, 3]
strings = ["a", "b", "c"]
nested = [[1, 2], [3, 4]]
empty = []
mixed = [1, "two", 3.0, true]
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }
}

mod keys {
    #[test]
    fn dotted_keys() {
        let table = shiguredo_toml::from_str(
            r#"
name = "Orange"
physical.color = "orange"
physical.shape = "round"
site."google.com" = true
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn quoted_keys() {
        let table = shiguredo_toml::from_str(
            r#"
"hello world" = 1
"" = "empty"
'literal key' = 2
"ʎǝʞ" = "unicode"
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }
}

mod complex {
    #[test]
    fn realistic_config() {
        let table = shiguredo_toml::from_str(
            r#"
# 設定ファイル例
title = "TOML Example"

[owner]
name = "Tom Preston-Werner"

[database]
enabled = true
ports = [8000, 8001, 8002]
data = [["delta", "phi"], [3.14]]
temp_targets = {cpu = 79.5, case = 72.0}

[[servers]]
name = "alpha"
ip = "10.0.0.1"
role = "frontend"

[[servers]]
name = "beta"
ip = "10.0.0.2"
role = "backend"
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn comment_preservation_parse() {
        // コメントはパース後の Value に含まれないことを確認
        let table = shiguredo_toml::from_str(
            r#"
# top comment
key = "value" # inline comment
# another comment
other = 42
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }

    #[test]
    fn super_table_implicit() {
        let table = shiguredo_toml::from_str(
            r#"
[a.b]
c = 1

[a]
d = 2
"#,
        )
        .expect("TOML のパースに成功するはず");
        insta::assert_debug_snapshot!(table);
    }
}
