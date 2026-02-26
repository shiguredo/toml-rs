use crate::TomlVersion;
use crate::error::Error;
use crate::parser;
use crate::span::{CommentIndex, PathSegment, SectionIndex, SpanIndex, TextSpan, parse_value_path};
use crate::value::{Table, Value};

/// 元テキストを保持しながら値を更新できる TOML ドキュメント。
#[derive(Debug, Clone)]
pub struct Document {
    source: String,
    table: Table,
    spans: SpanIndex,
    comments: CommentIndex,
    sections: SectionIndex,
}

impl Document {
    /// TOML 文字列から編集可能ドキュメントを作成する。
    pub fn parse(input: &str) -> Result<Self, Error> {
        let (table, spans, comments, sections) =
            parser::parse_with_spans(input, TomlVersion::V1_0)?;
        Ok(Self {
            source: input.to_owned(),
            table,
            spans,
            comments,
            sections,
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

    /// セクション位置インデックスを返す。
    pub fn sections(&self) -> &SectionIndex {
        &self.sections
    }

    /// 指定パスの値を置換、またはパスが存在しなければ新規挿入する。
    ///
    /// 中間テーブルが存在しない場合はエラーを返す。
    /// 更新後に全体を再解析し、位置情報も更新する。
    pub fn set(&mut self, path: &[PathSegment], new_value: Value) -> Result<(), Error> {
        if let Some(span) = self.spans.get(path) {
            // 既存値の置換
            let replacement = crate::to_inline_string(&new_value)?;

            if span.start > span.end || span.end > self.source.len() {
                return Err(Error::serialize("invalid value span"));
            }

            let mut next_source = self.source.clone();
            next_source.replace_range(span.start..span.end, &replacement);

            self.reparse(next_source)
        } else {
            // 新規挿入
            self.insert_at_path(path, new_value)
        }
    }

    /// 文字列パスで指定した値を置換、またはパスが存在しなければ新規挿入する。
    pub fn set_path(&mut self, path: &str, new_value: Value) -> Result<(), Error> {
        let parsed = parse_value_path(path)
            .map_err(|msg| Error::serialize(format!("failed to parse value path: {msg}")))?;
        self.set(&parsed, new_value)
    }

    /// ソーステキストを再パースして内部状態を更新する。
    fn reparse(&mut self, next_source: String) -> Result<(), Error> {
        let (next_table, next_spans, next_comments, next_sections) =
            parser::parse_with_spans(&next_source, TomlVersion::V1_0)?;
        self.source = next_source;
        self.table = next_table;
        self.spans = next_spans;
        self.comments = next_comments;
        self.sections = next_sections;
        Ok(())
    }

    /// パスが存在しない場合に新規キー値ペアを挿入する。
    fn insert_at_path(&mut self, path: &[PathSegment], new_value: Value) -> Result<(), Error> {
        // 末尾セグメントが Index の場合はエラー（配列要素の追加は非対応）
        let last = path.last().ok_or_else(|| Error::serialize("empty path"))?;
        match last {
            PathSegment::Key(_) => {}
            PathSegment::Index(_) => {
                return Err(Error::serialize("cannot insert an array element via set"));
            }
        };

        let parent_path = &path[..path.len() - 1];

        // 親がインラインテーブルかどうかを判定する
        if !parent_path.is_empty()
            && let Some(parent_span) = self.spans.get(parent_path)
        {
            let parent_text = &self.source[parent_span.start..parent_span.end];
            if parent_text.starts_with('{') {
                return self.insert_into_inline_table(path, new_value);
            }
        }

        // セクションテーブルまたはルートへの挿入
        let key = match last {
            PathSegment::Key(key) => key,
            _ => unreachable!(),
        };
        let inline_value = crate::to_inline_string(&new_value)?;
        let key_text = format_key(key);
        let insert_text = format!("{key_text} = {inline_value}\n");

        let insert_pos = self.find_insert_position(parent_path)?;

        let mut next_source = self.source.clone();
        next_source.insert_str(insert_pos, &insert_text);

        self.reparse(next_source)
    }

    /// 親パスに基づいてセクションテーブルまたはルートへの挿入位置を決定する。
    fn find_insert_position(&self, parent_path: &[PathSegment]) -> Result<usize, Error> {
        if parent_path.is_empty() {
            // ルートレベルへの挿入
            let pos = strip_trailing_blank_lines(&self.source, self.sections.root_end);
            return Ok(pos);
        }

        // 親が存在するか確認する
        let parent_value = value_at_path(&self.table, parent_path)
            .ok_or_else(|| Error::serialize("parent table does not exist"))?;

        match parent_value {
            Value::Table(_) => {}
            _ => {
                return Err(Error::serialize("parent path does not point to a table"));
            }
        }

        // セクションテーブル: SectionIndex からセクションの body_end に挿入する
        // body_end は次のセクションヘッダ直前を指すため、末尾の空行を除いた位置に挿入する
        if let Some(section_span) = self.sections.get(parent_path) {
            let pos = strip_trailing_blank_lines(&self.source, section_span.body_end);
            return Ok(pos);
        }

        Err(Error::serialize(
            "cannot determine insert position for the parent table",
        ))
    }

    /// インラインテーブル内への新規キー挿入テキストを生成する。
    fn insert_into_inline_table(
        &mut self,
        path: &[PathSegment],
        new_value: Value,
    ) -> Result<(), Error> {
        let last = path.last().ok_or_else(|| Error::serialize("empty path"))?;
        let key = match last {
            PathSegment::Key(key) => key,
            PathSegment::Index(_) => {
                return Err(Error::serialize("cannot insert an array element via set"));
            }
        };

        let parent_path = &path[..path.len() - 1];
        let parent_span = self
            .spans
            .get(parent_path)
            .ok_or_else(|| Error::serialize("parent span not found"))?;

        let parent_value = value_at_path(&self.table, parent_path)
            .ok_or_else(|| Error::serialize("parent table does not exist"))?;
        let parent_table = parent_value
            .as_table()
            .ok_or_else(|| Error::serialize("parent is not a table"))?;

        let inline_value = crate::to_inline_string(&new_value)?;
        let key_text = format_key(key);

        // 閉じ } の位置
        let close_brace_pos = self.source[..parent_span.end]
            .rfind('}')
            .ok_or_else(|| Error::serialize("inline table closing brace not found"))?;

        // 閉じ } の直前にスペースがあるかチェックして、整形を保つ
        let has_space_before_brace =
            close_brace_pos > 0 && self.source.as_bytes()[close_brace_pos - 1] == b' ';

        let insert_text = if parent_table.is_empty() {
            format!("{key_text} = {inline_value}")
        } else if has_space_before_brace {
            // "{ a = 1 }" -> "{ a = 1, b = 2 }"
            // スペースの前に挿入するので、スペースは維持される
            format!(", {key_text} = {inline_value}")
        } else {
            format!(", {key_text} = {inline_value}")
        };

        // スペースがある場合はスペースの前に挿入する
        let insert_pos = if has_space_before_brace {
            close_brace_pos - 1
        } else {
            close_brace_pos
        };

        let mut next_source = self.source.clone();
        next_source.insert_str(insert_pos, &insert_text);

        self.reparse(next_source)
    }
}

impl std::str::FromStr for Document {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// TOML キーを適切にフォーマットする（必要に応じてクォートする）。
fn format_key(key: &str) -> String {
    if key.is_empty()
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        // クォートが必要
        let mut out = String::with_capacity(key.len() + 2);
        out.push('"');
        for ch in key.chars() {
            match ch {
                '\u{0008}' => out.push_str("\\b"),
                '\t' => out.push_str("\\t"),
                '\n' => out.push_str("\\n"),
                '\u{000C}' => out.push_str("\\f"),
                '\r' => out.push_str("\\r"),
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                c if c.is_control() => {
                    let code = c as u32;
                    if code <= 0xFFFF {
                        out.push_str(&format!("\\u{code:04X}"));
                    } else {
                        out.push_str(&format!("\\U{code:08X}"));
                    }
                }
                c => out.push(c),
            }
        }
        out.push('"');
        out
    } else {
        key.to_owned()
    }
}

/// セクション body_end から末尾の空行（空白のみの行を含む）を逆方向にスキップし、
/// 最後の有効な行の直後の位置を返す。
fn strip_trailing_blank_lines(source: &str, body_end: usize) -> usize {
    let bytes = source.as_bytes();
    let mut pos = body_end;

    // 末尾の空行を逆方向にスキップする
    while pos > 0 && bytes[pos - 1] == b'\n' {
        // 改行の直前をスキャンして、行の内容が空白のみかどうかを判定する
        let line_end = pos - 1;
        let mut line_start = line_end;
        while line_start > 0 && bytes[line_start - 1] != b'\n' {
            line_start -= 1;
        }
        // 行の内容が空白のみであれば空行とみなしてスキップする
        let line_content = &bytes[line_start..line_end];
        if line_content.iter().all(|&b| b == b' ' || b == b'\t') {
            pos = line_start;
        } else {
            break;
        }
    }

    pos
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
