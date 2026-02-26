use shiguredo_toml::{Document, PathSegment, Value};

mod insert_output {
    use super::*;

    #[test]
    fn insert_keys_at_various_positions() {
        let input = r#"
title = "example"
port = 8080 # keep

[server]
host = "localhost"

[[items]]
name = "alpha"
[[items]]
name = "beta"
"#
        .trim_start();

        let mut doc = Document::parse(input).unwrap();
        doc.set_path("version", Value::Integer(1)).unwrap();
        doc.set_path("server.timeout", Value::Integer(30)).unwrap();
        doc.set_path("items[1].priority", Value::Integer(5))
            .unwrap();

        insta::assert_snapshot!(doc.as_str());
    }

    #[test]
    fn insert_into_inline_table() {
        let input = "config = { debug = true }\n";
        let mut doc = Document::parse(input).unwrap();
        doc.set_path("config.verbose", Value::Boolean(false))
            .unwrap();

        insta::assert_snapshot!(doc.as_str());
    }

    #[test]
    fn insert_into_section_with_trailing_blank_line() {
        let input = "[env]\nFOO = \"BAR\"\n\n[target.aarch64-unknown-linux-gnu]\n";
        let mut doc = Document::parse(input).unwrap();
        doc.set_path("env.PIYO", Value::String("FUGA".to_owned()))
            .unwrap();

        insta::assert_snapshot!(doc.as_str());
    }
}

mod edit_output {
    use super::*;

    #[test]
    fn replace_nested_values() {
        let input = r#"
title = "example"
port = 8080 # keep
arr = [1, { nested = true }]

[[servers]]
name = "alpha" # first
[[servers]]
name = "beta" # second
"#
        .trim_start();

        let mut doc = Document::parse(input).unwrap();
        doc.set_path("port", Value::Integer(9090)).unwrap();
        doc.set_path("arr[1].nested", Value::Boolean(false))
            .unwrap();
        doc.set_path("servers[1].name", Value::String("gamma".to_owned()))
            .unwrap();

        insta::assert_snapshot!(doc.as_str());
    }
}

mod positions {
    use super::*;

    #[test]
    fn value_and_comment_positions() {
        let input = r#"
title = "あ"
port = 8080 # keep
arr = [1, { nested = true }] # arr

[[servers]]
name = "alpha" # first
[[servers]]
name = "beta" # second
# orphan
"#
        .trim_start();

        let doc = Document::parse(input).unwrap();

        let snapshot = format!(
            "[value spans]\n{}\n\n[comment spans]\n{}",
            format_value_spans(&doc),
            format_comment_spans(&doc),
        );
        insta::assert_snapshot!(snapshot);
    }

    fn format_value_spans(doc: &Document) -> String {
        let mut lines: Vec<String> = doc
            .spans()
            .iter()
            .map(|(path, span)| {
                let path = format_path(path);
                let text = &doc.as_str()[span.start..span.end];
                format!("{path}@{}..{}={text:?}", span.start, span.end)
            })
            .collect();
        lines.sort();
        lines.join("\n")
    }

    fn format_comment_spans(doc: &Document) -> String {
        let mut entries: Vec<(usize, String)> = doc
            .comments()
            .iter()
            .map(|comment| {
                let target = comment
                    .target
                    .as_ref()
                    .map(|path| format_path(path))
                    .unwrap_or_else(|| "-".to_owned());
                let text = &doc.as_str()[comment.span.start..comment.span.end];
                (
                    comment.span.start,
                    format!(
                        "{target}@{}..{}={text:?}",
                        comment.span.start, comment.span.end
                    ),
                )
            })
            .collect();

        entries.sort_by_key(|entry| entry.0);
        entries
            .into_iter()
            .map(|entry| entry.1)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_path(path: &[PathSegment]) -> String {
        let mut out = String::new();

        for segment in path {
            match segment {
                PathSegment::Key(key) => {
                    if !out.is_empty() {
                        out.push('.');
                    }
                    out.push_str(key);
                }
                PathSegment::Index(index) => {
                    out.push('[');
                    out.push_str(&index.to_string());
                    out.push(']');
                }
            }
        }

        out
    }
}
