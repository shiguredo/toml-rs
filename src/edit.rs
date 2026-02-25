use crate::error::Error;
use crate::parser;
use crate::span::{CommentIndex, PathSegment, SpanIndex, TextSpan, parse_value_path};
use crate::value::{Table, Value};

/// 元テキストを保持しながら値を更新できる TOML ドキュメント。
#[derive(Debug, Clone)]
pub struct Document {
    source: String,
    table: Table,
    spans: SpanIndex,
    comments: CommentIndex,
}

impl Document {
    /// TOML 文字列から編集可能ドキュメントを作成する。
    pub fn parse(input: &str) -> Result<Self, Error> {
        let (table, spans, comments) = parser::parse_with_spans(input)?;
        Ok(Self {
            source: input.to_owned(),
            table,
            spans,
            comments,
        })
    }

    /// 現在の TOML テキストを返す。
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// 現在のルートテーブルを返す。
    pub fn as_table(&self) -> &Table {
        &self.table
    }

    /// 値位置インデックスを返す。
    pub fn spans(&self) -> &SpanIndex {
        &self.spans
    }

    /// コメント位置インデックスを返す。
    pub fn comments(&self) -> &CommentIndex {
        &self.comments
    }

    /// 指定パスの範囲を返す。
    pub fn span(&self, path: &[PathSegment]) -> Option<TextSpan> {
        self.spans.get(path)
    }

    /// 指定パスに紐づく行末コメント範囲を返す。
    pub fn trailing_comment_span(&self, path: &[PathSegment]) -> Option<TextSpan> {
        self.comments.trailing_for(path)
    }

    /// 文字列パスで指定した行末コメント範囲を返す。
    pub fn trailing_comment_span_path(&self, path: &str) -> Option<TextSpan> {
        let parsed = parse_value_path(path).ok()?;
        self.trailing_comment_span(&parsed)
    }

    /// 指定パスの値を返す。
    pub fn get(&self, path: &[PathSegment]) -> Option<&Value> {
        value_at_path(&self.table, path)
    }

    /// 文字列パスで指定した値を返す。
    pub fn get_path(&self, path: &str) -> Option<&Value> {
        let parsed = parse_value_path(path).ok()?;
        self.get(&parsed)
    }

    /// 指定パスの既存値を置換する。
    ///
    /// 置換後に全体を再解析し、位置情報も更新する。
    pub fn set(&mut self, path: &[PathSegment], new_value: Value) -> Result<(), Error> {
        let span = self
            .spans
            .get(path)
            .ok_or_else(|| Error::serialize("指定パスの値が見つからない"))?;
        let replacement = crate::to_inline_string(&new_value)?;

        if span.start > span.end || span.end > self.source.len() {
            return Err(Error::serialize("値範囲が不正"));
        }

        let mut next_source = self.source.clone();
        next_source.replace_range(span.start..span.end, &replacement);

        let (next_table, next_spans, next_comments) = parser::parse_with_spans(&next_source)?;
        self.source = next_source;
        self.table = next_table;
        self.spans = next_spans;
        self.comments = next_comments;
        Ok(())
    }

    /// 文字列パスで指定した既存値を置換する。
    pub fn set_path(&mut self, path: &str, new_value: Value) -> Result<(), Error> {
        let parsed = parse_value_path(path)
            .map_err(|msg| Error::serialize(format!("値パスの解析に失敗: {msg}")))?;
        self.set(&parsed, new_value)
    }
}

impl std::str::FromStr for Document {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

fn value_at_path<'a>(table: &'a Table, path: &[PathSegment]) -> Option<&'a Value> {
    let (first, rest) = path.split_first()?;
    let mut current = match first {
        PathSegment::Key(key) => table.get(key)?,
        PathSegment::Index(_) => return None,
    };

    for segment in rest {
        match segment {
            PathSegment::Key(key) => {
                current = current.as_table()?.get(key)?;
            }
            PathSegment::Index(index) => {
                current = current.as_array()?.get(*index)?;
            }
        }
    }

    Some(current)
}
