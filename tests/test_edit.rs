use shiguredo_toml::{Document, PathSegment, Value, parse_value_path};

mod parse_path {
    use super::*;

    #[test]
    fn parse_with_array_index() {
        let path = parse_value_path("servers[1].port").unwrap();
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
        let doc = Document::parse(input).unwrap();

        let port_path = parse_value_path("server.port").unwrap();
        let port_span = doc.span(&port_path).unwrap();
        assert_eq!(&input[port_span.start..port_span.end], "8080");

        let nested_path = parse_value_path("server.arr[1].nested").unwrap();
        let nested_span = doc.span(&nested_path).unwrap();
        assert_eq!(&input[nested_span.start..nested_span.end], "true");
    }
}

mod edit_value {
    use super::*;

    #[test]
    fn replace_scalar_preserves_around_text() {
        let input = "port = 8080 # keep comment\n";
        let mut doc = Document::parse(input).unwrap();

        doc.set_path("port", Value::Integer(9090)).unwrap();
        assert_eq!(doc.as_str(), "port = 9090 # keep comment\n");
    }

    #[test]
    fn replace_nested_inline_table_value() {
        let input = "arr = [1, { nested = true }]\n";
        let mut doc = Document::parse(input).unwrap();

        doc.set_path("arr[1].nested", Value::Boolean(false))
            .unwrap();
        assert_eq!(doc.as_str(), "arr = [1, { nested = false }]\n");
    }

    #[test]
    fn set_unknown_path_returns_error() {
        let input = "a = 1\n";
        let mut doc = Document::parse(input).unwrap();

        let result = doc.set_path("missing.key", Value::Integer(2));
        assert!(result.is_err());
    }
}

mod comment_tracking {
    use super::*;

    #[test]
    fn trailing_comment_is_tracked_by_value_path() {
        let input = "port = 8080 # keep comment\n";
        let doc = Document::parse(input).unwrap();

        let comment_span = doc.trailing_comment_span_path("port").unwrap();
        assert_eq!(
            &input[comment_span.start..comment_span.end],
            "# keep comment"
        );
    }

    #[test]
    fn utf8_value_and_comment_spans_are_byte_exact() {
        let input = "msg = \"あ\"\nport = 8080 # コメント\n";
        let doc = Document::parse(input).unwrap();

        let value_span = doc.span(&parse_value_path("msg").unwrap()).unwrap();
        assert_eq!(&input[value_span.start..value_span.end], "\"あ\"");

        let comment_span = doc.trailing_comment_span_path("port").unwrap();
        assert_eq!(&input[comment_span.start..comment_span.end], "# コメント");
    }

    #[test]
    fn array_of_tables_comments_follow_indexed_paths() {
        let input = "[[servers]]\nport = 8080 # first\n\n[[servers]]\nport = 9090 # second\n";
        let doc = Document::parse(input).unwrap();

        let first = doc.trailing_comment_span_path("servers[0].port").unwrap();
        assert_eq!(&input[first.start..first.end], "# first");

        let second = doc.trailing_comment_span_path("servers[1].port").unwrap();
        assert_eq!(&input[second.start..second.end], "# second");
    }

    #[test]
    fn trailing_comment_span_is_updated_after_edit() {
        let input = "port = 8 # keep\n";
        let mut doc = Document::parse(input).unwrap();

        let before = doc.trailing_comment_span_path("port").unwrap();
        doc.set_path("port", Value::Integer(123456)).unwrap();
        let after = doc.trailing_comment_span_path("port").unwrap();

        assert_eq!(doc.as_str(), "port = 123456 # keep\n");
        assert_eq!(&doc.as_str()[after.start..after.end], "# keep");
        assert!(after.start > before.start);
    }

    #[test]
    fn comment_only_line_is_recorded_without_target() {
        let input = "# only comment\na = 1\n";
        let doc = Document::parse(input).unwrap();

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
