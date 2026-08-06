use shiguredo_toml::{Document, PathSegment, Value, parse_value_path};

mod parse_path {
    use super::*;

    #[test]
    fn parse_with_array_index() {
        let path = parse_value_path("servers[1].port").expect("パスのパースに成功するはず");
        assert_eq!(
            path,
            vec![
                PathSegment::Key("servers".to_owned()),
                PathSegment::Index(1),
                PathSegment::Key("port".to_owned()),
            ]
        );
    }
}

mod span_tracking {
    use super::*;

    #[test]
    fn span_for_nested_values() {
        let input = "title = \"x\"\n[server]\nport = 8080\narr = [1, { nested = true }]\n";
        let doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let port_path = parse_value_path("server.port").expect("パスのパースに成功するはず");
        let port_span = doc.span(&port_path).expect("スパンは存在するはず");
        assert_eq!(&input[port_span.start..port_span.end], "8080");

        let nested_path =
            parse_value_path("server.arr[1].nested").expect("パスのパースに成功するはず");
        let nested_span = doc.span(&nested_path).expect("スパンは存在するはず");
        assert_eq!(&input[nested_span.start..nested_span.end], "true");
    }
}

mod edit_value {
    use super::*;

    #[test]
    fn replace_scalar_preserves_around_text() {
        let input = "port = 8080 # keep comment\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("port", Value::Integer(9090))
            .expect("編集に成功するはず");
        assert_eq!(doc.as_str(), "port = 9090 # keep comment\n");
    }

    #[test]
    fn replace_nested_inline_table_value() {
        let input = "arr = [1, { nested = true }]\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("arr[1].nested", Value::Boolean(false))
            .expect("編集に成功するはず");
        assert_eq!(doc.as_str(), "arr = [1, { nested = false }]\n");
    }

    #[test]
    fn set_existing_key_still_replaces() {
        let input = "port = 8080\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("port", Value::Integer(9090))
            .expect("編集に成功するはず");
        assert_eq!(doc.as_str(), "port = 9090\n");
        assert_eq!(
            doc.get_path("port")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            9090
        );
    }
}

mod insert_value {
    use super::*;

    #[test]
    fn insert_new_key_at_root() {
        let input = "a = 1\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("b", Value::Integer(2))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("b")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            2
        );
        // 元のキーは保持される
        assert_eq!(
            doc.get_path("a")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            1
        );
    }

    #[test]
    fn insert_new_key_at_root_empty_doc() {
        let input = "";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("key", Value::String("value".to_owned()))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("key")
                .expect("パスは存在するはず")
                .as_str()
                .expect("値は文字列になるはず"),
            "value"
        );
    }

    #[test]
    fn insert_new_key_at_root_without_trailing_newline() {
        let input = r#"env = "test""#;
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("target", Value::String("value".to_owned()))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("target")
                .expect("パスは存在するはず")
                .as_str()
                .expect("値は文字列になるはず"),
            "value"
        );
        assert_eq!(
            doc.get_path("env")
                .expect("パスは存在するはず")
                .as_str()
                .expect("値は文字列になるはず"),
            "test"
        );
    }

    #[test]
    fn insert_new_key_in_section() {
        let input = "[server]\nport = 8080\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("server.host", Value::String("localhost".to_owned()))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("server.host")
                .expect("パスは存在するはず")
                .as_str()
                .expect("値は文字列になるはず"),
            "localhost"
        );
        // 元のキーは保持される
        assert_eq!(
            doc.get_path("server.port")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            8080
        );
    }

    #[test]
    fn insert_new_key_in_array_table_element() {
        let input = "[[servers]]\nname = \"alpha\"\n[[servers]]\nname = \"beta\"\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("servers[1].port", Value::Integer(9090))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("servers[1].port")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            9090
        );
        // 元のキーは保持される
        assert_eq!(
            doc.get_path("servers[1].name")
                .expect("パスは存在するはず")
                .as_str()
                .expect("値は文字列になるはず"),
            "beta"
        );
    }

    #[test]
    fn insert_new_key_in_inline_table() {
        let input = "obj = { a = 1 }\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("obj.b", Value::Integer(2))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("obj.b")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            2
        );
        // 元のキーは保持される
        assert_eq!(
            doc.get_path("obj.a")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            1
        );
    }

    #[test]
    fn insert_new_key_in_empty_inline_table() {
        let input = "obj = {}\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("obj.x", Value::Boolean(true))
            .expect("編集に成功するはず");
        assert!(
            doc.get_path("obj.x")
                .expect("パスは存在するはず")
                .as_bool()
                .expect("値はブール値になるはず")
        );
    }

    #[test]
    fn insert_with_scalar_parent_returns_error() {
        let input = "a = 1\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let result = doc.set_path("a.child", Value::Integer(2));
        assert!(result.is_err());
    }

    #[test]
    fn insert_array_element_returns_error() {
        let input = "arr = [1, 2]\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let result = doc.set(
            &[PathSegment::Key("arr".to_owned()), PathSegment::Index(5)],
            Value::Integer(3),
        );
        assert!(result.is_err());
    }

    #[test]
    fn insert_preserves_comments() {
        let input = "# header\na = 1 # keep\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("b", Value::Integer(2))
            .expect("編集に成功するはず");

        // コメントが保持されているか確認
        assert!(doc.as_str().contains("# header"));
        assert!(doc.as_str().contains("# keep"));
        assert_eq!(
            doc.get_path("b")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            2
        );
    }

    #[test]
    fn insert_table_value_as_inline() {
        let input = "[server]\nport = 8080\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let mut table = shiguredo_toml::Table::new();
        table.insert("x".to_owned(), Value::Integer(1));
        table.insert("y".to_owned(), Value::Integer(2));
        doc.set_path("server.pos", Value::Table(table))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("server.pos.x")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            1
        );
    }

    #[test]
    fn insert_into_section_with_following_section() {
        let input = "[a]\nx = 1\n[b]\ny = 2\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("a.z", Value::Integer(99))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("a.z")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            99
        );
        // 他のセクションは保持される
        assert_eq!(
            doc.get_path("b.y")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            2
        );
    }
}

mod comment_tracking {
    use super::*;

    #[test]
    fn trailing_comment_is_tracked_by_value_path() {
        let input = "port = 8080 # keep comment\n";
        let doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let comment_span = doc
            .trailing_comment_span_path("port")
            .expect("コメントスパンは存在するはず");
        assert_eq!(
            &input[comment_span.start..comment_span.end],
            "# keep comment"
        );
    }

    #[test]
    fn utf8_value_and_comment_spans_are_byte_exact() {
        let input = "msg = \"あ\"\nport = 8080 # コメント\n";
        let doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let value_span = doc
            .span(&parse_value_path("msg").expect("スパンは存在するはず"))
            .expect("スパンは存在するはず");
        assert_eq!(&input[value_span.start..value_span.end], "\"あ\"");

        let comment_span = doc
            .trailing_comment_span_path("port")
            .expect("コメントスパンは存在するはず");
        assert_eq!(&input[comment_span.start..comment_span.end], "# コメント");
    }

    #[test]
    fn array_of_tables_comments_follow_indexed_paths() {
        let input = "[[servers]]\nport = 8080 # first\n\n[[servers]]\nport = 9090 # second\n";
        let doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let first = doc
            .trailing_comment_span_path("servers[0].port")
            .expect("コメントスパンは存在するはず");
        assert_eq!(&input[first.start..first.end], "# first");

        let second = doc
            .trailing_comment_span_path("servers[1].port")
            .expect("コメントスパンは存在するはず");
        assert_eq!(&input[second.start..second.end], "# second");
    }

    #[test]
    fn trailing_comment_span_is_updated_after_edit() {
        let input = "port = 8 # keep\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let before = doc
            .trailing_comment_span_path("port")
            .expect("コメントスパンは存在するはず");
        doc.set_path("port", Value::Integer(123456))
            .expect("編集に成功するはず");
        let after = doc
            .trailing_comment_span_path("port")
            .expect("コメントスパンは存在するはず");

        assert_eq!(doc.as_str(), "port = 123456 # keep\n");
        assert_eq!(&doc.as_str()[after.start..after.end], "# keep");
        assert!(after.start > before.start);
    }

    #[test]
    fn comment_only_line_is_recorded_without_target() {
        let input = "# only comment\na = 1\n";
        let doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let comments: Vec<_> = doc
            .comments()
            .iter()
            .filter(|comment| comment.target.is_none())
            .collect();
        assert_eq!(comments.len(), 1);
        let span = comments[0].span;
        assert_eq!(&input[span.start..span.end], "# only comment");
    }
}

mod insert_auto_create {
    use super::*;

    #[test]
    fn creates_section_for_missing_parent() {
        let input = "a = 1\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("new.key", Value::Integer(2))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("new.key")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            2
        );
        assert_eq!(
            doc.get_path("a")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            1
        );
    }

    #[test]
    fn creates_section_in_existing_section() {
        let input = "[server]\nport = 8080\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("server.db.host", Value::String("localhost".into()))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("server.db.host")
                .expect("パスは存在するはず")
                .as_str()
                .expect("値は文字列になるはず"),
            "localhost"
        );
        assert_eq!(
            doc.get_path("server.port")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            8080
        );
    }

    #[test]
    fn creates_deep_nested_section() {
        let input = "";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("a.b.c.key", Value::Integer(42))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("a.b.c.key")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            42
        );
    }

    #[test]
    fn subsequent_insert_into_auto_created_section() {
        let input = "[server]\nport = 8080\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("server.db.host", Value::String("localhost".into()))
            .expect("編集に成功するはず");
        doc.set_path("server.db.port", Value::Integer(5432))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("server.db.host")
                .expect("パスは存在するはず")
                .as_str()
                .expect("値は文字列になるはず"),
            "localhost"
        );
        assert_eq!(
            doc.get_path("server.db.port")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            5432
        );
    }

    #[test]
    fn auto_create_with_following_section() {
        let input = "[a]\nx = 1\n\n[b]\ny = 2\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        doc.set_path("a.sub.key", Value::Integer(99))
            .expect("編集に成功するはず");
        assert_eq!(
            doc.get_path("a.sub.key")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            99
        );
        assert_eq!(
            doc.get_path("b.y")
                .expect("パスは存在するはず")
                .as_integer()
                .expect("値は整数になるはず"),
            2
        );
    }

    #[test]
    fn scalar_intermediate_returns_error() {
        let input = "a = 1\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let result = doc.set_path("a.b.c", Value::Integer(2));
        assert!(result.is_err());
    }

    #[test]
    fn array_index_in_missing_portion_returns_error() {
        let input = "[server]\nport = 8080\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let result = doc.set(
            &[
                PathSegment::Key("server".into()),
                PathSegment::Key("items".into()),
                PathSegment::Index(0),
                PathSegment::Key("name".into()),
            ],
            Value::String("x".into()),
        );
        assert!(result.is_err());
    }
}

mod datetime_validate {
    use super::*;
    use shiguredo_toml::{Datetime, Offset};

    /// 無効な Datetime（4 バリアントに該当しない組み合わせ）。
    fn invalid_datetime() -> Datetime {
        Datetime {
            date: None,
            time: None,
            offset: Some(Offset::Z),
        }
    }

    /// エラーが Error::Serialize であることを検証する。
    fn assert_serialize_error(result: Result<(), shiguredo_toml::Error>) {
        assert!(
            matches!(result, Err(shiguredo_toml::Error::Serialize { .. })),
            "Error::Serialize になるはず"
        );
    }

    /// 既存キーの置換で無効な Datetime を渡すと Err になり、
    /// ドキュメントの内容が変化しない。
    #[test]
    fn replace_existing_key_returns_error_and_keeps_source() {
        let input = "port = 8080\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let result = doc.set_path("port", Value::Datetime(invalid_datetime()));
        assert_serialize_error(result);
        assert_eq!(doc.as_str(), input);
    }

    /// `set`（PathSegment 版）で既存キーの置換に無効な Datetime を渡すと
    /// Err になり、ドキュメントの内容が変化しない。
    #[test]
    fn set_with_path_segments_returns_error_and_keeps_source() {
        let input = "port = 8080\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let path = parse_value_path("port").expect("パスのパースに成功するはず");
        let result = doc.set(&path, Value::Datetime(invalid_datetime()));
        assert_serialize_error(result);
        assert_eq!(doc.as_str(), input);
    }

    /// 新規キーの挿入で無効な Datetime を渡すと Err になり、
    /// ドキュメントの内容が変化しない。
    #[test]
    fn insert_new_key_returns_error_and_keeps_source() {
        let input = "a = 1\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let result = doc.set_path("b", Value::Datetime(invalid_datetime()));
        assert_serialize_error(result);
        assert_eq!(doc.as_str(), input);
    }

    /// インラインテーブルへの挿入で無効な Datetime を渡すと Err になり、
    /// ドキュメントの内容が変化しない。
    #[test]
    fn insert_into_inline_table_returns_error_and_keeps_source() {
        let input = "obj = { a = 1 }\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let result = doc.set_path("obj.b", Value::Datetime(invalid_datetime()));
        assert_serialize_error(result);
        assert_eq!(doc.as_str(), input);
    }

    /// セクションの自動生成を伴う挿入で無効な Datetime を渡すと Err になり、
    /// ドキュメントの内容が変化しない。
    #[test]
    fn insert_with_new_section_returns_error_and_keeps_source() {
        let input = "a = 1\n";
        let mut doc = Document::parse(input).expect("TOML のパースに成功するはず");

        let result = doc.set_path("new.key", Value::Datetime(invalid_datetime()));
        assert_serialize_error(result);
        assert_eq!(doc.as_str(), input);
    }
}
