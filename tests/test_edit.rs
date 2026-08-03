use shiguredo_toml::{Document, PathSegment, Value, parse_value_path};

mod parse_path {
    use super::*;

    #[test]
    fn parse_with_array_index() {
        let path = parse_value_path("servers[1].port").expect("path should parse");
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
        let doc = Document::parse(input).expect("TOML should parse");

        let port_path = parse_value_path("server.port").expect("path should parse");
        let port_span = doc.span(&port_path).expect("span should exist");
        assert_eq!(&input[port_span.start..port_span.end], "8080");

        let nested_path = parse_value_path("server.arr[1].nested").expect("path should parse");
        let nested_span = doc.span(&nested_path).expect("span should exist");
        assert_eq!(&input[nested_span.start..nested_span.end], "true");
    }
}

mod edit_value {
    use super::*;

    #[test]
    fn replace_scalar_preserves_around_text() {
        let input = "port = 8080 # keep comment\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("port", Value::Integer(9090))
            .expect("edit should succeed");
        assert_eq!(doc.as_str(), "port = 9090 # keep comment\n");
    }

    #[test]
    fn replace_nested_inline_table_value() {
        let input = "arr = [1, { nested = true }]\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("arr[1].nested", Value::Boolean(false))
            .expect("edit should succeed");
        assert_eq!(doc.as_str(), "arr = [1, { nested = false }]\n");
    }

    #[test]
    fn set_existing_key_still_replaces() {
        let input = "port = 8080\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("port", Value::Integer(9090))
            .expect("edit should succeed");
        assert_eq!(doc.as_str(), "port = 9090\n");
        assert_eq!(
            doc.get_path("port")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            9090
        );
    }
}

mod insert_value {
    use super::*;

    #[test]
    fn insert_new_key_at_root() {
        let input = "a = 1\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("b", Value::Integer(2))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("b")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            2
        );
        // 元のキーは保持される
        assert_eq!(
            doc.get_path("a")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            1
        );
    }

    #[test]
    fn insert_new_key_at_root_empty_doc() {
        let input = "";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("key", Value::String("value".to_owned()))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("key")
                .expect("path should exist")
                .as_str()
                .expect("value should be a string"),
            "value"
        );
    }

    #[test]
    fn insert_new_key_at_root_without_trailing_newline() {
        let input = r#"env = "test""#;
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("target", Value::String("value".to_owned()))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("target")
                .expect("path should exist")
                .as_str()
                .expect("value should be a string"),
            "value"
        );
        assert_eq!(
            doc.get_path("env")
                .expect("path should exist")
                .as_str()
                .expect("value should be a string"),
            "test"
        );
    }

    #[test]
    fn insert_new_key_in_section() {
        let input = "[server]\nport = 8080\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("server.host", Value::String("localhost".to_owned()))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("server.host")
                .expect("path should exist")
                .as_str()
                .expect("value should be a string"),
            "localhost"
        );
        // 元のキーは保持される
        assert_eq!(
            doc.get_path("server.port")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            8080
        );
    }

    #[test]
    fn insert_new_key_in_array_table_element() {
        let input = "[[servers]]\nname = \"alpha\"\n[[servers]]\nname = \"beta\"\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("servers[1].port", Value::Integer(9090))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("servers[1].port")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            9090
        );
        // 元のキーは保持される
        assert_eq!(
            doc.get_path("servers[1].name")
                .expect("path should exist")
                .as_str()
                .expect("value should be a string"),
            "beta"
        );
    }

    #[test]
    fn insert_new_key_in_inline_table() {
        let input = "obj = { a = 1 }\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("obj.b", Value::Integer(2))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("obj.b")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            2
        );
        // 元のキーは保持される
        assert_eq!(
            doc.get_path("obj.a")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            1
        );
    }

    #[test]
    fn insert_new_key_in_empty_inline_table() {
        let input = "obj = {}\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("obj.x", Value::Boolean(true))
            .expect("edit should succeed");
        assert!(
            doc.get_path("obj.x")
                .expect("path should exist")
                .as_bool()
                .expect("value should be a boolean")
        );
    }

    #[test]
    fn insert_with_scalar_parent_returns_error() {
        let input = "a = 1\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        let result = doc.set_path("a.child", Value::Integer(2));
        assert!(result.is_err());
    }

    #[test]
    fn insert_array_element_returns_error() {
        let input = "arr = [1, 2]\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        let result = doc.set(
            &[PathSegment::Key("arr".to_owned()), PathSegment::Index(5)],
            Value::Integer(3),
        );
        assert!(result.is_err());
    }

    #[test]
    fn insert_preserves_comments() {
        let input = "# header\na = 1 # keep\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("b", Value::Integer(2))
            .expect("edit should succeed");

        // コメントが保持されているか確認
        assert!(doc.as_str().contains("# header"));
        assert!(doc.as_str().contains("# keep"));
        assert_eq!(
            doc.get_path("b")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            2
        );
    }

    #[test]
    fn insert_table_value_as_inline() {
        let input = "[server]\nport = 8080\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        let mut table = shiguredo_toml::Table::new();
        table.insert("x".to_owned(), Value::Integer(1));
        table.insert("y".to_owned(), Value::Integer(2));
        doc.set_path("server.pos", Value::Table(table))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("server.pos.x")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            1
        );
    }

    #[test]
    fn insert_into_section_with_following_section() {
        let input = "[a]\nx = 1\n[b]\ny = 2\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("a.z", Value::Integer(99))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("a.z")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            99
        );
        // 他のセクションは保持される
        assert_eq!(
            doc.get_path("b.y")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            2
        );
    }
}

mod comment_tracking {
    use super::*;

    #[test]
    fn trailing_comment_is_tracked_by_value_path() {
        let input = "port = 8080 # keep comment\n";
        let doc = Document::parse(input).expect("TOML should parse");

        let comment_span = doc
            .trailing_comment_span_path("port")
            .expect("comment span should exist");
        assert_eq!(
            &input[comment_span.start..comment_span.end],
            "# keep comment"
        );
    }

    #[test]
    fn utf8_value_and_comment_spans_are_byte_exact() {
        let input = "msg = \"あ\"\nport = 8080 # コメント\n";
        let doc = Document::parse(input).expect("TOML should parse");

        let value_span = doc
            .span(&parse_value_path("msg").expect("span should exist"))
            .expect("span should exist");
        assert_eq!(&input[value_span.start..value_span.end], "\"あ\"");

        let comment_span = doc
            .trailing_comment_span_path("port")
            .expect("comment span should exist");
        assert_eq!(&input[comment_span.start..comment_span.end], "# コメント");
    }

    #[test]
    fn array_of_tables_comments_follow_indexed_paths() {
        let input = "[[servers]]\nport = 8080 # first\n\n[[servers]]\nport = 9090 # second\n";
        let doc = Document::parse(input).expect("TOML should parse");

        let first = doc
            .trailing_comment_span_path("servers[0].port")
            .expect("comment span should exist");
        assert_eq!(&input[first.start..first.end], "# first");

        let second = doc
            .trailing_comment_span_path("servers[1].port")
            .expect("comment span should exist");
        assert_eq!(&input[second.start..second.end], "# second");
    }

    #[test]
    fn trailing_comment_span_is_updated_after_edit() {
        let input = "port = 8 # keep\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        let before = doc
            .trailing_comment_span_path("port")
            .expect("comment span should exist");
        doc.set_path("port", Value::Integer(123456))
            .expect("edit should succeed");
        let after = doc
            .trailing_comment_span_path("port")
            .expect("comment span should exist");

        assert_eq!(doc.as_str(), "port = 123456 # keep\n");
        assert_eq!(&doc.as_str()[after.start..after.end], "# keep");
        assert!(after.start > before.start);
    }

    #[test]
    fn comment_only_line_is_recorded_without_target() {
        let input = "# only comment\na = 1\n";
        let doc = Document::parse(input).expect("TOML should parse");

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
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("new.key", Value::Integer(2))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("new.key")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            2
        );
        assert_eq!(
            doc.get_path("a")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            1
        );
    }

    #[test]
    fn creates_section_in_existing_section() {
        let input = "[server]\nport = 8080\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("server.db.host", Value::String("localhost".into()))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("server.db.host")
                .expect("path should exist")
                .as_str()
                .expect("value should be a string"),
            "localhost"
        );
        assert_eq!(
            doc.get_path("server.port")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            8080
        );
    }

    #[test]
    fn creates_deep_nested_section() {
        let input = "";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("a.b.c.key", Value::Integer(42))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("a.b.c.key")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            42
        );
    }

    #[test]
    fn subsequent_insert_into_auto_created_section() {
        let input = "[server]\nport = 8080\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("server.db.host", Value::String("localhost".into()))
            .expect("edit should succeed");
        doc.set_path("server.db.port", Value::Integer(5432))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("server.db.host")
                .expect("path should exist")
                .as_str()
                .expect("value should be a string"),
            "localhost"
        );
        assert_eq!(
            doc.get_path("server.db.port")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            5432
        );
    }

    #[test]
    fn auto_create_with_following_section() {
        let input = "[a]\nx = 1\n\n[b]\ny = 2\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        doc.set_path("a.sub.key", Value::Integer(99))
            .expect("edit should succeed");
        assert_eq!(
            doc.get_path("a.sub.key")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            99
        );
        assert_eq!(
            doc.get_path("b.y")
                .expect("path should exist")
                .as_integer()
                .expect("value should be an integer"),
            2
        );
    }

    #[test]
    fn scalar_intermediate_returns_error() {
        let input = "a = 1\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

        let result = doc.set_path("a.b.c", Value::Integer(2));
        assert!(result.is_err());
    }

    #[test]
    fn array_index_in_missing_portion_returns_error() {
        let input = "[server]\nport = 8080\n";
        let mut doc = Document::parse(input).expect("TOML should parse");

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
