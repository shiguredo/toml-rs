use std::collections::{HashMap, HashSet};

use crate::TomlVersion;
use crate::datetime;
use crate::error::Error;
use crate::span::{
    CommentIndex, PathSegment, SectionIndex, SectionSpan, SpanIndex, TextSpan, ValuePath,
};
use crate::value::{Array, Table, Value};

/// ネストの最大深さ。
const MAX_DEPTH: usize = 128;

/// テーブルの定義方法を追跡する。
#[derive(Clone, Copy, PartialEq)]
enum TableState {
    /// `[header]` で明示定義されたテーブル
    Explicit,
    /// dotted key により定義されたテーブル（`[header]` での再定義不可）
    Dotted,
    /// dotted key の中間パスまたは `[header]` のスーパーテーブルとして暗黙作成
    Implicit,
    /// インラインテーブル `{ }` で定義（キー追加不可）
    Inline,
}

/// TOML パーサ。
struct Parser<'a> {
    /// 元の入力全体
    input: &'a str,
    /// 残りの未パース部分（input のスライス）
    rest: &'a str,
    /// ルートテーブル
    root: Table,
    /// 現在のテーブルパス（`[header]` で設定される）
    current_path: Vec<String>,
    /// テーブルパス -> 定義状態のマップ
    table_states: HashMap<Vec<String>, TableState>,
    /// 配列テーブルとして定義されたパスの集合
    array_table_paths: HashSet<Vec<String>>,
    /// 配列テーブルパスごとの現在要素インデックス
    array_table_current_index: HashMap<Vec<String>, usize>,
    /// 現在解析中の値パス
    value_path_stack: ValuePath,
    /// 値パスと元テキスト範囲
    span_index: SpanIndex,
    /// コメント位置情報
    comment_index: CommentIndex,
    /// 次に現れる行末コメントの紐づけ先
    pending_comment_target: Option<ValuePath>,
    /// セクション範囲インデックス
    section_index: SectionIndex,
    /// 現在のセクションパス（None はルートセクション）
    current_section_path: Option<ValuePath>,
    /// 現在のセクション本文の開始バイト位置
    current_section_body_start: usize,
    /// 現在のネスト深さ
    depth: usize,
    /// パース対象の TOML バージョン
    version: TomlVersion,
}

/// TOML 文字列を解析して Table に変換する。
pub(crate) fn parse(input: &str, version: TomlVersion) -> Result<Table, Error> {
    parse_with_spans(input, version).map(|(table, _, _, _)| table)
}

/// TOML 文字列を解析して Table と位置情報に変換する。
pub(crate) fn parse_with_spans(
    input: &str,
    version: TomlVersion,
) -> Result<(Table, SpanIndex, CommentIndex, SectionIndex), Error> {
    let mut parser = Parser {
        input,
        rest: input,
        root: Table::new(),
        current_path: Vec::new(),
        table_states: HashMap::new(),
        array_table_paths: HashSet::new(),
        array_table_current_index: HashMap::new(),
        value_path_stack: Vec::new(),
        span_index: SpanIndex::new(),
        comment_index: CommentIndex::new(),
        pending_comment_target: None,
        section_index: SectionIndex::new(),
        current_section_path: None,
        current_section_body_start: 0,
        depth: 0,
        version,
    };
    parser.parse_document()?;
    Ok((
        parser.root,
        parser.span_index,
        parser.comment_index,
        parser.section_index,
    ))
}

impl<'a> Parser<'a> {
    /// 現在のバイト位置を返す。
    fn position(&self) -> usize {
        self.input.len() - self.rest.len()
    }

    /// エラーを生成する。
    fn error(&self, message: impl Into<String>) -> Error {
        Error::parse(self.position(), message)
    }

    /// 先頭バイトを覗く。
    fn peek(&self) -> Option<u8> {
        self.rest.as_bytes().first().copied()
    }

    /// 先頭から n バイト先を覗く。
    fn peek_at(&self, n: usize) -> Option<u8> {
        self.rest.as_bytes().get(n).copied()
    }

    /// 先頭の文字を消費して返す。
    fn advance_char(&mut self) -> Option<char> {
        let mut chars = self.rest.chars();
        let ch = chars.next()?;
        self.rest = chars.as_str();
        Some(ch)
    }

    /// 先頭の n バイトを消費する。
    fn advance_bytes(&mut self, n: usize) {
        self.rest = &self.rest[n..];
    }

    /// 入力終端かどうかを返す。
    fn at_eof(&self) -> bool {
        self.rest.is_empty()
    }

    /// 空白（タブとスペース）をスキップする。改行はスキップしない。
    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek() {
            if b == b' ' || b == b'\t' {
                self.advance_bytes(1);
            } else {
                break;
            }
        }
    }

    /// 改行を消費する。LF または CRLF。
    fn consume_newline(&mut self) -> bool {
        if self.rest.starts_with("\r\n") {
            self.advance_bytes(2);
            true
        } else if self.rest.starts_with('\n') {
            self.advance_bytes(1);
            true
        } else {
            false
        }
    }

    /// コメントをスキップする。
    fn skip_comment(&mut self) -> Result<(), Error> {
        if self.peek() != Some(b'#') {
            return Ok(());
        }
        let start = self.position();
        self.advance_bytes(1);
        while !self.at_eof() {
            let b = self.peek().unwrap();
            if b == b'\n' || b == b'\r' {
                break;
            }
            // 制御文字チェック（タブは許可）
            if b != b'\t' && (b <= 0x08 || (0x0A..=0x1F).contains(&b) || b == 0x7F) {
                return Err(self.error(format!("invalid control character in comment: U+{b:04X}")));
            }
            // UTF-8 文字として進める
            self.advance_char();
        }
        let end = self.position();
        let target = self.pending_comment_target.take();
        self.comment_index.insert(TextSpan::new(start, end), target);
        Ok(())
    }

    /// 空白、コメント、改行をスキップする（配列内で使用）。
    fn skip_whitespace_comment_newline(&mut self) -> Result<(), Error> {
        loop {
            self.skip_whitespace();
            if self.peek() == Some(b'#') {
                self.skip_comment()?;
            }
            if !self.consume_newline() {
                break;
            }
        }
        Ok(())
    }

    /// 指定バイトを期待して消費する。
    fn expect(&mut self, expected: u8) -> Result<(), Error> {
        match self.peek() {
            Some(b) if b == expected => {
                self.advance_bytes(1);
                Ok(())
            }
            Some(b) => Err(self.error(format!(
                "expected '{}' but found '{}'",
                expected as char, b as char
            ))),
            None => Err(self.error(format!("expected '{}' but input ended", expected as char))),
        }
    }

    /// 改行または EOF を期待する。
    fn expect_newline_or_eof(&mut self) -> Result<(), Error> {
        if self.at_eof() || self.consume_newline() {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected newline or EOF but found '{}'",
                self.rest.chars().next().unwrap_or('?')
            )))
        }
    }

    // ========== ドキュメント解析 ==========

    /// TOML ドキュメント全体を解析する。
    fn parse_document(&mut self) -> Result<(), Error> {
        loop {
            self.pending_comment_target = None;
            self.skip_whitespace();
            if self.at_eof() {
                break;
            }
            match self.peek() {
                Some(b'#') => {
                    self.skip_comment()?;
                    if !self.at_eof() {
                        self.expect_newline_or_eof()?;
                    }
                }
                Some(b'\n') => {
                    self.consume_newline();
                }
                Some(b'\r') => {
                    if !self.consume_newline() {
                        return Err(self.error("invalid newline: TOML v1.0.0 requires LF or CRLF"));
                    }
                }
                Some(b'[') => {
                    // 前のセクションの body_end を確定する
                    self.finalize_current_section();
                    self.parse_table_header()?;
                    self.skip_whitespace();
                    if self.peek() == Some(b'#') {
                        self.skip_comment()?;
                    }
                    if !self.at_eof() {
                        self.expect_newline_or_eof()?;
                    }
                    // 新しいセクションの本文開始位置を記録する
                    self.current_section_body_start = self.position();
                }
                Some(_) => {
                    self.parse_keyval()?;
                    self.skip_whitespace();
                    if self.peek() == Some(b'#') {
                        self.skip_comment()?;
                    }
                    if !self.at_eof() {
                        self.expect_newline_or_eof()?;
                    }
                }
                None => break,
            }
        }
        // 最後のセクションの body_end を確定する
        self.finalize_current_section();
        Ok(())
    }

    // ========== セクション範囲追跡 ==========

    /// 現在のセクションの body_end を確定し、SectionIndex に登録する。
    fn finalize_current_section(&mut self) {
        let body_end = self.position();
        match self.current_section_path.take() {
            Some(path) => {
                self.section_index.insert(
                    path,
                    SectionSpan {
                        body_start: self.current_section_body_start,
                        body_end,
                    },
                );
            }
            None => {
                // ルートセクション
                self.section_index.root_end = body_end;
            }
        }
    }

    // ========== テーブルヘッダ解析 ==========

    /// `[table]` または `[[array]]` ヘッダを解析する。
    fn parse_table_header(&mut self) -> Result<(), Error> {
        let header_pos = self.position();
        self.expect(b'[')?;
        let is_array = self.peek() == Some(b'[');
        if is_array {
            self.advance_bytes(1);
        }

        self.skip_whitespace();
        let path = self.parse_key()?;
        self.skip_whitespace();

        if is_array {
            self.expect(b']')?;
            self.expect(b']')?;
            self.handle_array_table(path, header_pos)?;
        } else {
            self.expect(b']')?;
            self.handle_standard_table(path, header_pos)?;
        }

        // 現在のセクションパスを設定する（配列テーブルの場合はインデックス付き）
        self.current_section_path = Some(self.current_context_path());

        Ok(())
    }

    /// 標準テーブル `[path]` を処理する。
    fn handle_standard_table(&mut self, path: Vec<String>, header_pos: usize) -> Result<(), Error> {
        let mut table = &mut self.root;

        for (i, part) in path.iter().enumerate() {
            let prefix_path = path[..=i].to_vec();
            let is_last = i == path.len() - 1;

            if self.array_table_paths.contains(&prefix_path) {
                if is_last {
                    return Err(Error::parse(
                        header_pos,
                        format!(
                            "'{}' is defined as an array table and cannot be a standard table",
                            path_to_string(&path)
                        ),
                    ));
                }
                // 配列テーブルの最後の要素にナビゲート
                let entry = table.get_mut(part).ok_or_else(|| {
                    Error::parse(
                        header_pos,
                        format!("internal error: array table '{part}' not found"),
                    )
                })?;
                let array = entry
                    .as_array_mut()
                    .ok_or_else(|| Error::parse(header_pos, format!("'{part}' is not an array")))?;
                let last = array.last_mut().ok_or_else(|| {
                    Error::parse(header_pos, format!("array table '{part}' is empty"))
                })?;
                table = last.as_table_mut().ok_or_else(|| {
                    Error::parse(
                        header_pos,
                        format!("element of array table '{part}' is not a table"),
                    )
                })?;
                continue;
            }

            if is_last {
                match self.table_states.get(&prefix_path) {
                    Some(TableState::Explicit) => {
                        return Err(Error::parse(
                            header_pos,
                            format!("table '{}' is already defined", path_to_string(&path)),
                        ));
                    }
                    Some(TableState::Dotted) => {
                        return Err(Error::parse(
                            header_pos,
                            format!(
                                "table '{}' is already defined by dotted key",
                                path_to_string(&path)
                            ),
                        ));
                    }
                    Some(TableState::Inline) => {
                        return Err(Error::parse(
                            header_pos,
                            format!(
                                "table '{}' is defined as an inline table and cannot be redefined",
                                path_to_string(&path)
                            ),
                        ));
                    }
                    Some(TableState::Implicit) | None => {
                        self.table_states.insert(prefix_path, TableState::Explicit);
                    }
                }

                if !table.contains_key(part) {
                    table.insert(part.clone(), Value::Table(Table::new()));
                }
                match table.get(part) {
                    Some(Value::Table(_)) => {}
                    Some(other) => {
                        return Err(Error::parse(
                            header_pos,
                            format!("key '{part}' is already defined as {}", other.type_name()),
                        ));
                    }
                    None => unreachable!(),
                }
            } else {
                if let Some(TableState::Inline) = self.table_states.get(&prefix_path) {
                    return Err(Error::parse(
                        header_pos,
                        format!(
                            "table '{}' is defined as an inline table and cannot be extended",
                            path_to_string(&prefix_path)
                        ),
                    ));
                }

                if !table.contains_key(part) {
                    table.insert(part.clone(), Value::Table(Table::new()));
                    self.table_states
                        .entry(prefix_path)
                        .or_insert(TableState::Implicit);
                }

                match table.get_mut(part) {
                    Some(Value::Table(t)) => table = t,
                    Some(other) => {
                        return Err(Error::parse(
                            header_pos,
                            format!("key '{part}' is already defined as {}", other.type_name()),
                        ));
                    }
                    None => unreachable!(),
                }
            }
        }

        self.current_path = path;
        Ok(())
    }

    /// 配列テーブル `[[path]]` を処理する。
    fn handle_array_table(&mut self, path: Vec<String>, header_pos: usize) -> Result<(), Error> {
        let mut table = &mut self.root;

        for (i, part) in path.iter().enumerate() {
            let prefix_path = path[..=i].to_vec();
            let is_last = i == path.len() - 1;

            if is_last {
                match self.table_states.get(&prefix_path) {
                    Some(TableState::Explicit | TableState::Dotted | TableState::Implicit) => {
                        if !self.array_table_paths.contains(&prefix_path) {
                            return Err(Error::parse(
                                header_pos,
                                format!(
                                    "'{}' is defined as a standard table and cannot be an array table",
                                    path_to_string(&path)
                                ),
                            ));
                        }
                    }
                    Some(TableState::Inline) => {
                        return Err(Error::parse(
                            header_pos,
                            format!(
                                "'{}' is defined as an inline table and cannot be an array table",
                                path_to_string(&path)
                            ),
                        ));
                    }
                    None => {}
                }

                if self.array_table_paths.contains(&prefix_path) {
                    if let Some(entry) = table.get_mut(part) {
                        let array = entry.as_array_mut().ok_or_else(|| {
                            Error::parse(header_pos, format!("'{part}' is not an array"))
                        })?;
                        array.push(Value::Table(Table::new()));
                        self.array_table_current_index
                            .insert(prefix_path.clone(), array.len() - 1);
                    } else {
                        // 新しい親要素のコンテキストではキーがまだ存在しない
                        let array = vec![Value::Table(Table::new())];
                        table.insert(part.clone(), Value::Array(array));
                        self.array_table_current_index
                            .insert(prefix_path.clone(), 0);
                    }
                    // サブパスのテーブル状態をリセットする
                    self.table_states
                        .retain(|k, _| !(k.len() > path.len() && k.starts_with(&path)));
                } else {
                    if let Some(existing) = table.get(part) {
                        if existing.is_array() {
                            return Err(Error::parse(
                                header_pos,
                                format!(
                                    "cannot append to static array '{}' using array table syntax",
                                    path_to_string(&path)
                                ),
                            ));
                        }
                        return Err(Error::parse(
                            header_pos,
                            format!(
                                "key '{part}' is already defined as {}",
                                existing.type_name()
                            ),
                        ));
                    }
                    let array = vec![Value::Table(Table::new())];
                    table.insert(part.clone(), Value::Array(array));
                    self.array_table_paths.insert(prefix_path.clone());
                    self.array_table_current_index.insert(prefix_path, 0);
                }
            } else {
                if self.array_table_paths.contains(&prefix_path) {
                    let entry = table.get_mut(part).ok_or_else(|| {
                        Error::parse(
                            header_pos,
                            format!("internal error: array table '{part}' not found"),
                        )
                    })?;
                    let array = entry.as_array_mut().ok_or_else(|| {
                        Error::parse(header_pos, format!("'{part}' is not an array"))
                    })?;
                    let last = array.last_mut().ok_or_else(|| {
                        Error::parse(header_pos, format!("array table '{part}' is empty"))
                    })?;
                    table = last.as_table_mut().ok_or_else(|| {
                        Error::parse(
                            header_pos,
                            format!("element of array table '{part}' is not a table"),
                        )
                    })?;
                    continue;
                }

                if let Some(TableState::Inline) = self.table_states.get(&prefix_path) {
                    return Err(Error::parse(
                        header_pos,
                        format!(
                            "table '{}' is defined as an inline table and cannot be extended",
                            path_to_string(&prefix_path)
                        ),
                    ));
                }

                if !table.contains_key(part) {
                    table.insert(part.clone(), Value::Table(Table::new()));
                    self.table_states
                        .entry(prefix_path)
                        .or_insert(TableState::Implicit);
                }

                match table.get_mut(part) {
                    Some(Value::Table(t)) => table = t,
                    Some(other) => {
                        return Err(Error::parse(
                            header_pos,
                            format!("key '{part}' is already defined as {}", other.type_name()),
                        ));
                    }
                    None => unreachable!(),
                }
            }
        }

        self.current_path = path;
        Ok(())
    }

    // ========== キー値ペア解析 ==========

    /// 現在のテーブルコンテキストを値パスとして返す。
    fn current_context_path(&self) -> ValuePath {
        let mut path = Vec::new();
        let mut prefix = Vec::new();

        for part in &self.current_path {
            prefix.push(part.clone());
            path.push(PathSegment::Key(part.clone()));
            if self.array_table_paths.contains(&prefix) {
                let index = *self.array_table_current_index.get(&prefix).unwrap_or(&0);
                path.push(PathSegment::Index(index));
            }
        }

        path
    }

    /// 現在のテーブルコンテキストとキーから値パスを作成する。
    fn make_value_path(&self, key_parts: &[String]) -> ValuePath {
        let mut path = self.current_context_path();
        for part in key_parts {
            path.push(PathSegment::Key(part.clone()));
        }
        path
    }

    /// キー値ペアを解析して現在のテーブルに挿入する。
    fn parse_keyval(&mut self) -> Result<(), Error> {
        let key_pos = self.position();
        let key_parts = self.parse_key()?;
        self.skip_whitespace();
        self.expect(b'=')?;
        self.skip_whitespace();

        let value_path = self.make_value_path(&key_parts);
        self.value_path_stack = value_path.clone();
        let value = self.parse_value()?;
        self.value_path_stack.clear();
        self.pending_comment_target = Some(value_path);

        // フィールド分割借用: root, current_path, array_table_paths, table_states を
        // 個別に借用してフリー関数に渡す。
        let path = self.current_path.clone();
        let table = navigate_table_mut(&mut self.root, &path, &self.array_table_paths, key_pos)?;
        insert_dotted_key(
            table,
            &key_parts,
            value,
            key_pos,
            &self.current_path,
            &mut self.table_states,
        )?;

        Ok(())
    }

    /// dotted key に従ってインラインテーブル内にキー値を挿入する。
    fn insert_dotted_key_inline(
        table: &mut Table,
        key_parts: &[String],
        value: Value,
        key_pos: usize,
        dotted_created_paths: &mut HashSet<Vec<String>>,
    ) -> Result<(), Error> {
        let mut current = table;

        for (i, part) in key_parts[..key_parts.len() - 1].iter().enumerate() {
            let path = key_parts[..=i].to_vec();
            if current.contains_key(part) {
                match current.get_mut(part) {
                    Some(Value::Table(t)) => {
                        if !dotted_created_paths.contains(&path) {
                            return Err(Error::parse(
                                key_pos,
                                format!(
                                    "key '{}' is already defined in inline table and cannot be extended with dotted key",
                                    part
                                ),
                            ));
                        }
                        current = t;
                    }
                    Some(other) => {
                        return Err(Error::parse(
                            key_pos,
                            format!(
                                "key '{part}' is already defined as {} and cannot be used as a table",
                                other.type_name()
                            ),
                        ));
                    }
                    None => unreachable!(),
                }
            } else {
                current.insert(part.clone(), Value::Table(Table::new()));
                dotted_created_paths.insert(path);
                current = current.get_mut(part).unwrap().as_table_mut().unwrap();
            }
        }

        let final_key = &key_parts[key_parts.len() - 1];
        if current.contains_key(final_key) {
            return Err(Error::parse(
                key_pos,
                format!("key '{final_key}' is already defined"),
            ));
        }
        current.insert(final_key.clone(), value);
        Ok(())
    }

    // ========== キー解析 ==========

    /// TOML キーを解析する（dotted key 対応）。
    fn parse_key(&mut self) -> Result<Vec<String>, Error> {
        let mut parts = Vec::new();
        parts.push(self.parse_simple_key()?);

        loop {
            let saved = self.rest;
            self.skip_whitespace();
            if self.peek() == Some(b'.') {
                self.advance_bytes(1);
                self.skip_whitespace();
                parts.push(self.parse_simple_key()?);
            } else {
                self.rest = saved;
                self.skip_whitespace();
                break;
            }
        }

        Ok(parts)
    }

    /// 単純キー（bare key または quoted key）を解析する。
    fn parse_simple_key(&mut self) -> Result<String, Error> {
        match self.peek() {
            Some(b'"') => {
                if self.rest.starts_with("\"\"\"") {
                    return Err(self.error("multiline strings cannot be used as keys"));
                }
                self.parse_basic_string()
            }
            Some(b'\'') => {
                if self.rest.starts_with("'''") {
                    return Err(self.error("multiline strings cannot be used as keys"));
                }
                self.parse_literal_string()
            }
            Some(b) if is_bare_key_char(b) => self.parse_bare_key(),
            Some(b) => Err(self.error(format!("invalid key start character: '{}'", b as char))),
            None => Err(self.error("expected key but input ended")),
        }
    }

    /// ベアキーを解析する。
    fn parse_bare_key(&mut self) -> Result<String, Error> {
        let start = self.rest;
        let mut len = 0;
        while let Some(&b) = self.rest.as_bytes().get(len) {
            if is_bare_key_char(b) {
                len += 1;
            } else {
                break;
            }
        }
        if len == 0 {
            return Err(self.error("empty bare key"));
        }
        let key = start[..len].to_owned();
        self.advance_bytes(len);
        Ok(key)
    }

    // ========== 値解析 ==========

    /// TOML 値を解析する。
    fn parse_value(&mut self) -> Result<Value, Error> {
        let start = self.position();
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.error(format!("nesting depth exceeds maximum of {MAX_DEPTH}")));
        }
        let result = self.parse_value_inner();
        self.depth -= 1;
        if result.is_ok() && !self.value_path_stack.is_empty() {
            let end = self.position();
            self.span_index
                .insert(self.value_path_stack.clone(), TextSpan::new(start, end));
        }
        result
    }

    fn parse_value_inner(&mut self) -> Result<Value, Error> {
        match self.peek() {
            Some(b'"') => {
                if self.rest.starts_with("\"\"\"") {
                    self.parse_ml_basic_string().map(Value::String)
                } else {
                    self.parse_basic_string().map(Value::String)
                }
            }
            Some(b'\'') => {
                if self.rest.starts_with("'''") {
                    self.parse_ml_literal_string().map(Value::String)
                } else {
                    self.parse_literal_string().map(Value::String)
                }
            }
            Some(b't') => self.parse_bool_true(),
            Some(b'f') => self.parse_bool_false(),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_inline_table(),
            Some(b'i') => self.parse_inf(false),
            Some(b'n') => self.parse_nan(false),
            Some(b'+') => match self.peek_at(1) {
                Some(b'i') => self.parse_inf(false),
                Some(b'n') => self.parse_nan(false),
                _ => self.parse_number_or_datetime(),
            },
            Some(b'-') => match self.peek_at(1) {
                Some(b'i') => self.parse_inf(true),
                Some(b'n') => self.parse_nan(true),
                _ => self.parse_number_or_datetime(),
            },
            Some(b) if b.is_ascii_digit() => self.parse_number_or_datetime(),
            Some(b) => Err(self.error(format!("invalid value start character: '{}'", b as char))),
            None => Err(self.error("expected value but input ended")),
        }
    }

    // ========== 文字列解析 ==========

    /// 基本文字列 `"..."` を解析する。
    fn parse_basic_string(&mut self) -> Result<String, Error> {
        self.expect(b'"')?;
        let mut result = String::new();

        loop {
            match self.peek() {
                Some(b'"') => {
                    self.advance_bytes(1);
                    return Ok(result);
                }
                Some(b'\\') => {
                    self.advance_bytes(1);
                    result.push(self.parse_escape_sequence()?);
                }
                Some(b'\n') | Some(b'\r') => {
                    return Err(self.error("newline not allowed in basic string"));
                }
                Some(b) if is_control_char(b) => {
                    return Err(self.error(format!(
                        "invalid control character in basic string: U+{b:04X}"
                    )));
                }
                Some(_) => {
                    let ch = self.advance_char().unwrap();
                    result.push(ch);
                }
                None => {
                    return Err(self.error("unterminated basic string"));
                }
            }
        }
    }

    /// 複数行基本文字列 `"""..."""` を解析する。
    fn parse_ml_basic_string(&mut self) -> Result<String, Error> {
        self.advance_bytes(3);

        // 開始デリミタ直後の改行は削除
        if self.rest.starts_with("\r\n") {
            self.advance_bytes(2);
        } else if self.rest.starts_with('\n') {
            self.advance_bytes(1);
        }

        let mut result = String::new();

        loop {
            if self.rest.starts_with("\"\"\"") {
                self.advance_bytes(3);
                // 終了デリミタの後に追加の引用符 ("""" or """""")
                let mut extra_quotes = 0;
                while self.peek() == Some(b'"') && extra_quotes < 2 {
                    result.push('"');
                    self.advance_bytes(1);
                    extra_quotes += 1;
                }
                return Ok(result);
            }

            match self.peek() {
                Some(b'\\') => {
                    self.advance_bytes(1);
                    if self.is_at_line_ending_backslash() {
                        self.skip_line_ending_whitespace();
                    } else {
                        result.push(self.parse_escape_sequence()?);
                    }
                }
                Some(b) if is_ml_basic_control_char(b) => {
                    return Err(self.error(format!(
                        "invalid control character in multiline basic string: U+{b:04X}"
                    )));
                }
                // V1_1: \r\n を \n に正規化する。\r 単体はエラー。
                Some(b'\r') if self.version == TomlVersion::V1_1 => {
                    self.advance_bytes(1);
                    if self.peek() != Some(b'\n') {
                        return Err(self.error(
                            "bare CR not allowed in multiline basic string in TOML v1.1.0",
                        ));
                    }
                    // \r\n の \r を消費。次のループで \n を処理する。
                }
                Some(_) => {
                    let ch = self.advance_char().unwrap();
                    result.push(ch);
                }
                None => {
                    return Err(self.error("unterminated multiline basic string"));
                }
            }
        }
    }

    /// line ending backslash の位置にいるかチェックする。
    fn is_at_line_ending_backslash(&self) -> bool {
        let bytes = self.rest.as_bytes();
        let mut i = 0;

        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }

        if i < bytes.len() && bytes[i] == b'\n' {
            return true;
        }
        if i + 1 < bytes.len() && bytes[i] == b'\r' && bytes[i + 1] == b'\n' {
            return true;
        }

        false
    }

    /// line ending backslash 以降の空白と改行をスキップする。
    fn skip_line_ending_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(b' ') | Some(b'\t') => self.advance_bytes(1),
                Some(b'\n') => {
                    self.advance_bytes(1);
                }
                Some(b'\r') if self.peek_at(1) == Some(b'\n') => {
                    self.advance_bytes(2);
                }
                _ => break,
            }
        }
    }

    /// リテラル文字列 `'...'` を解析する。
    fn parse_literal_string(&mut self) -> Result<String, Error> {
        self.expect(b'\'')?;
        let start = self.rest;
        let mut len = 0;

        loop {
            match self.rest.as_bytes().get(len) {
                Some(&b'\'') => {
                    let result = start[..len].to_owned();
                    self.advance_bytes(len + 1);
                    return Ok(result);
                }
                Some(&b'\n') | Some(&b'\r') => {
                    return Err(self.error("newline not allowed in literal string"));
                }
                Some(&b) if is_control_char(b) => {
                    self.advance_bytes(len);
                    return Err(self.error(format!(
                        "invalid control character in literal string: U+{b:04X}"
                    )));
                }
                Some(_) => {
                    let ch_len = utf8_char_len(self.rest.as_bytes()[len]);
                    len += ch_len;
                }
                None => {
                    return Err(self.error("unterminated literal string"));
                }
            }
        }
    }

    /// 複数行リテラル文字列 `'''...'''` を解析する。
    fn parse_ml_literal_string(&mut self) -> Result<String, Error> {
        self.advance_bytes(3);

        // 開始デリミタ直後の改行は削除
        if self.rest.starts_with("\r\n") {
            self.advance_bytes(2);
        } else if self.rest.starts_with('\n') {
            self.advance_bytes(1);
        }

        let mut result = String::new();

        loop {
            if self.rest.starts_with("'''") {
                self.advance_bytes(3);
                let mut extra_apos = 0;
                while self.peek() == Some(b'\'') && extra_apos < 2 {
                    result.push('\'');
                    self.advance_bytes(1);
                    extra_apos += 1;
                }
                return Ok(result);
            }

            match self.peek() {
                Some(b) if b != b'\t' && b != b'\n' && b != b'\r' && is_control_char(b) => {
                    return Err(self.error(format!(
                        "invalid control character in multiline literal string: U+{b:04X}"
                    )));
                }
                // V1_1: \r\n を \n に正規化する。\r 単体はエラー。
                Some(b'\r') if self.version == TomlVersion::V1_1 => {
                    self.advance_bytes(1);
                    if self.peek() != Some(b'\n') {
                        return Err(self.error(
                            "bare CR not allowed in multiline literal string in TOML v1.1.0",
                        ));
                    }
                    // \r\n の \r を消費。次のループで \n を処理する。
                }
                Some(_) => {
                    let ch = self.advance_char().unwrap();
                    result.push(ch);
                }
                None => {
                    return Err(self.error("unterminated multiline literal string"));
                }
            }
        }
    }

    /// エスケープシーケンスを解析する（バックスラッシュは消費済み）。
    fn parse_escape_sequence(&mut self) -> Result<char, Error> {
        match self.peek() {
            Some(b'b') => {
                self.advance_bytes(1);
                Ok('\u{0008}')
            }
            Some(b't') => {
                self.advance_bytes(1);
                Ok('\t')
            }
            Some(b'n') => {
                self.advance_bytes(1);
                Ok('\n')
            }
            Some(b'f') => {
                self.advance_bytes(1);
                Ok('\u{000C}')
            }
            Some(b'r') => {
                self.advance_bytes(1);
                Ok('\r')
            }
            Some(b'"') => {
                self.advance_bytes(1);
                Ok('"')
            }
            Some(b'\\') => {
                self.advance_bytes(1);
                Ok('\\')
            }
            Some(b'u') => {
                self.advance_bytes(1);
                self.parse_unicode_escape(4)
            }
            Some(b'U') => {
                self.advance_bytes(1);
                self.parse_unicode_escape(8)
            }
            // V1_1 追加: \e (ESC, U+001B)
            Some(b'e') if self.version == TomlVersion::V1_1 => {
                self.advance_bytes(1);
                Ok('\u{001B}')
            }
            // V1_1 追加: \xHH (2 桁 16 進数)
            Some(b'x') if self.version == TomlVersion::V1_1 => {
                self.advance_bytes(1);
                self.parse_unicode_escape(2)
            }
            Some(b) => Err(self.error(format!("invalid escape sequence: '\\{}'", b as char))),
            None => Err(self.error("incomplete escape sequence")),
        }
    }

    /// Unicode エスケープ `\uXXXX` または `\UXXXXXXXX` を解析する。
    fn parse_unicode_escape(&mut self, digit_count: usize) -> Result<char, Error> {
        if self.rest.len() < digit_count {
            return Err(self.error(format!(
                "Unicode escape requires {digit_count} hex digits but input is too short"
            )));
        }

        let hex_str = &self.rest[..digit_count];
        if !hex_str.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(self.error(format!("invalid hex digits in Unicode escape: '{hex_str}'")));
        }

        let code_point = u32::from_str_radix(hex_str, 16)
            .map_err(|e| self.error(format!("Unicode escape conversion error: {e}")))?;

        self.advance_bytes(digit_count);

        char::from_u32(code_point)
            .ok_or_else(|| self.error(format!("invalid Unicode scalar value: U+{code_point:04X}")))
    }

    // ========== ブール値解析 ==========

    fn parse_bool_true(&mut self) -> Result<Value, Error> {
        if self.rest.starts_with("true") {
            self.advance_bytes(4);
            Ok(Value::Boolean(true))
        } else {
            Err(self.error("expected 'true'"))
        }
    }

    fn parse_bool_false(&mut self) -> Result<Value, Error> {
        if self.rest.starts_with("false") {
            self.advance_bytes(5);
            Ok(Value::Boolean(false))
        } else {
            Err(self.error("expected 'false'"))
        }
    }

    // ========== 特殊浮動小数点値 ==========

    fn parse_inf(&mut self, negative: bool) -> Result<Value, Error> {
        if negative {
            if self.rest.starts_with("-inf") {
                self.advance_bytes(4);
                return Ok(Value::Float(f64::NEG_INFINITY));
            }
        } else if self.rest.starts_with("+inf") {
            self.advance_bytes(4);
            return Ok(Value::Float(f64::INFINITY));
        } else if self.rest.starts_with("inf") {
            self.advance_bytes(3);
            return Ok(Value::Float(f64::INFINITY));
        }
        Err(self.error("expected 'inf'"))
    }

    fn parse_nan(&mut self, negative: bool) -> Result<Value, Error> {
        if negative {
            if self.rest.starts_with("-nan") {
                self.advance_bytes(4);
                return Ok(Value::Float(f64::NAN));
            }
        } else if self.rest.starts_with("+nan") {
            self.advance_bytes(4);
            return Ok(Value::Float(f64::NAN));
        } else if self.rest.starts_with("nan") {
            self.advance_bytes(3);
            return Ok(Value::Float(f64::NAN));
        }
        Err(self.error("expected 'nan'"))
    }

    // ========== 数値/日時解析 ==========

    /// 数値または日時を解析する。
    fn parse_number_or_datetime(&mut self) -> Result<Value, Error> {
        let bytes = self.rest.as_bytes();
        let offset = usize::from(bytes.first() == Some(&b'+') || bytes.first() == Some(&b'-'));

        // 4 桁の数字の後に '-' があれば日付
        if offset == 0 && bytes.len() >= 5 {
            let has_4_digits = bytes[..4].iter().all(|b| b.is_ascii_digit());
            if has_4_digits && bytes[4] == b'-' {
                return self.parse_datetime_value();
            }
        }

        // 2 桁の数字の後に ':' があればローカル時刻
        if offset == 0 && bytes.len() >= 3 {
            let has_2_digits = bytes[..2].iter().all(|b| b.is_ascii_digit());
            if has_2_digits && bytes[2] == b':' {
                return self.parse_datetime_value();
            }
        }

        self.parse_number()
    }

    /// 数値（整数または浮動小数点数）を解析する。
    fn parse_number(&mut self) -> Result<Value, Error> {
        let start_pos = self.position();
        let start = self.rest;

        // 符号
        let has_sign = matches!(self.peek(), Some(b'+') | Some(b'-'));
        if has_sign {
            self.advance_bytes(1);
        }

        // プレフィックスチェック (符号なしの場合のみ)
        if !has_sign {
            if self.rest.starts_with("0x") {
                self.advance_bytes(2);
                return self.parse_hex_int(start_pos);
            }
            if self.rest.starts_with("0o") {
                self.advance_bytes(2);
                return self.parse_oct_int(start_pos);
            }
            if self.rest.starts_with("0b") {
                self.advance_bytes(2);
                return self.parse_bin_int(start_pos);
            }
        }

        // 10 進整数部分を読む
        let int_len = scan_digits(self.rest);
        if int_len == 0 {
            return Err(Error::parse(start_pos, "expected number"));
        }

        let int_part = &self.rest[..int_len];

        // 先頭ゼロチェック
        if int_part.as_bytes()[0] == b'0' && int_len > 1 {
            let second = int_part.as_bytes()[1];
            if second.is_ascii_digit() || second == b'_' {
                return Err(Error::parse(start_pos, "leading zeros are not allowed"));
            }
        }

        self.advance_bytes(int_len);

        // 浮動小数点の判定
        if self.peek() == Some(b'.') {
            if !matches!(self.peek_at(1), Some(b) if b.is_ascii_digit()) {
                return Err(Error::parse(
                    self.position(),
                    "expected digit after decimal point",
                ));
            }
            self.advance_bytes(1); // '.'
            let frac_len = scan_digits(self.rest);
            if frac_len == 0 {
                return Err(Error::parse(
                    self.position(),
                    "expected digit after decimal point",
                ));
            }
            self.advance_bytes(frac_len);

            if matches!(self.peek(), Some(b'e') | Some(b'E')) {
                self.advance_bytes(1);
                if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                    self.advance_bytes(1);
                }
                let exp_len = scan_digits(self.rest);
                if exp_len == 0 {
                    return Err(Error::parse(self.position(), "expected digit in exponent"));
                }
                self.advance_bytes(exp_len);
            }

            let raw = &start[..start.len() - self.rest.len()];
            return self.parse_float_from_raw(raw, start_pos);
        }

        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            self.advance_bytes(1);
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.advance_bytes(1);
            }
            let exp_len = scan_digits(self.rest);
            if exp_len == 0 {
                return Err(Error::parse(self.position(), "expected digit in exponent"));
            }
            self.advance_bytes(exp_len);

            let raw = &start[..start.len() - self.rest.len()];
            return self.parse_float_from_raw(raw, start_pos);
        }

        // 整数
        let full_raw = &start[..start.len() - self.rest.len()];
        // 数字部分のアンダースコア検証
        validate_underscore_rules(int_part, start_pos)?;

        let cleaned: String = full_raw.chars().filter(|c| *c != '_').collect();
        let n = cleaned
            .parse::<i64>()
            .map_err(|e| Error::parse(start_pos, format!("integer conversion error: {e}")))?;
        Ok(Value::Integer(n))
    }

    /// 16 進整数を解析する（`0x` は消費済み）。
    fn parse_hex_int(&mut self, start_pos: usize) -> Result<Value, Error> {
        let hex_start = self.rest;
        let len = scan_hex_digits(self.rest);
        if len == 0 {
            return Err(Error::parse(start_pos, "expected hex digit"));
        }
        let raw = &hex_start[..len];
        validate_underscore_rules(raw, start_pos)?;
        self.advance_bytes(len);

        let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
        let n = i64::from_str_radix(&cleaned, 16)
            .map_err(|e| Error::parse(start_pos, format!("hex integer conversion error: {e}")))?;
        Ok(Value::Integer(n))
    }

    /// 8 進整数を解析する（`0o` は消費済み）。
    fn parse_oct_int(&mut self, start_pos: usize) -> Result<Value, Error> {
        let oct_start = self.rest;
        let len = scan_oct_digits(self.rest);
        if len == 0 {
            return Err(Error::parse(start_pos, "expected octal digit"));
        }
        let raw = &oct_start[..len];
        validate_underscore_rules(raw, start_pos)?;
        self.advance_bytes(len);

        let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
        let n = i64::from_str_radix(&cleaned, 8)
            .map_err(|e| Error::parse(start_pos, format!("octal integer conversion error: {e}")))?;
        Ok(Value::Integer(n))
    }

    /// 2 進整数を解析する（`0b` は消費済み）。
    fn parse_bin_int(&mut self, start_pos: usize) -> Result<Value, Error> {
        let bin_start = self.rest;
        let len = scan_bin_digits(self.rest);
        if len == 0 {
            return Err(Error::parse(start_pos, "expected binary digit"));
        }
        let raw = &bin_start[..len];
        validate_underscore_rules(raw, start_pos)?;
        self.advance_bytes(len);

        let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
        let n = i64::from_str_radix(&cleaned, 2).map_err(|e| {
            Error::parse(start_pos, format!("binary integer conversion error: {e}"))
        })?;
        Ok(Value::Integer(n))
    }

    /// 浮動小数点数の生文字列をパースする。
    fn parse_float_from_raw(&self, raw: &str, start_pos: usize) -> Result<Value, Error> {
        // 各部分のアンダースコアルールを検証
        for part in raw.split(['.', 'e', 'E', '+', '-']) {
            if !part.is_empty() {
                validate_underscore_rules(part, start_pos)?;
            }
        }

        let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
        let f: f64 = cleaned
            .parse::<f64>()
            .map_err(|e| Error::parse(start_pos, format!("float conversion error: {e}")))?;
        Ok(Value::Float(f))
    }

    // ========== 日時解析 ==========

    /// 日時値を解析する。
    fn parse_datetime_value(&mut self) -> Result<Value, Error> {
        let start_pos = self.position();
        let start = self.rest;

        // 値の終わりまでスキャン
        let mut len = 0;
        while len < self.rest.len() {
            let b = self.rest.as_bytes()[len];
            if b == b' '
                || b == b'\t'
                || b == b'#'
                || b == b','
                || b == b']'
                || b == b'}'
                || b == b'\n'
                || b == b'\r'
            {
                // ただし日時の T の代わりの空白は含める
                // YYYY-MM-DD HH:MM:SS のパターン
                if b == b' ' && len == 10 {
                    // 日付の後の空白 -> 時刻が続くかチェック
                    if len + 1 < self.rest.len() && self.rest.as_bytes()[len + 1].is_ascii_digit() {
                        len += 1;
                        continue;
                    }
                }
                break;
            }
            len += 1;
        }

        let datetime_str = &start[..len];
        let dt = datetime::parse_datetime_str_with_version(datetime_str, self.version)
            .map_err(|e| Error::parse(start_pos, e.to_string()))?;

        self.advance_bytes(len);
        Ok(Value::Datetime(dt))
    }

    // ========== 配列解析 ==========

    /// TOML 配列 `[...]` を解析する。
    fn parse_array(&mut self) -> Result<Value, Error> {
        self.expect(b'[')?;
        let mut result = Array::new();

        self.skip_whitespace_comment_newline()?;

        if self.peek() == Some(b']') {
            self.advance_bytes(1);
            return Ok(Value::Array(result));
        }

        loop {
            self.skip_whitespace_comment_newline()?;
            let index = result.len();
            self.value_path_stack.push(PathSegment::Index(index));
            let value = self.parse_value()?;
            self.value_path_stack.pop();
            result.push(value);
            self.skip_whitespace_comment_newline()?;

            match self.peek() {
                Some(b',') => {
                    self.advance_bytes(1);
                    self.skip_whitespace_comment_newline()?;
                    if self.peek() == Some(b']') {
                        self.advance_bytes(1);
                        return Ok(Value::Array(result));
                    }
                }
                Some(b']') => {
                    self.advance_bytes(1);
                    return Ok(Value::Array(result));
                }
                _ => {
                    return Err(self.error("expected ',' or ']' in array"));
                }
            }
        }
    }

    // ========== インラインテーブル解析 ==========

    /// インラインテーブル `{...}` を解析する。
    ///
    /// v1.0.0: 単一行のみ、末尾カンマ禁止。
    /// v1.1.0: 複数行・末尾カンマ許可。
    fn parse_inline_table(&mut self) -> Result<Value, Error> {
        self.expect(b'{')?;
        let mut result = Table::new();
        let mut dotted_created_paths: HashSet<Vec<String>> = HashSet::new();

        self.skip_inline_whitespace()?;

        if self.peek() == Some(b'}') {
            self.advance_bytes(1);
            return Ok(Value::Table(result));
        }

        loop {
            self.skip_inline_whitespace()?;
            let key_pos = self.position();
            let key_parts = self.parse_key()?;
            self.skip_inline_whitespace()?;
            self.expect(b'=')?;
            self.skip_inline_whitespace()?;
            for part in &key_parts {
                self.value_path_stack.push(PathSegment::Key(part.clone()));
            }
            let value = self.parse_value()?;
            for _ in &key_parts {
                self.value_path_stack.pop();
            }

            Self::insert_dotted_key_inline(
                &mut result,
                &key_parts,
                value,
                key_pos,
                &mut dotted_created_paths,
            )?;

            self.skip_inline_whitespace()?;

            match self.peek() {
                Some(b',') => {
                    self.advance_bytes(1);
                    self.skip_inline_whitespace()?;
                    if self.peek() == Some(b'}') {
                        if self.version == TomlVersion::V1_0 {
                            return Err(Error::parse(
                                self.position(),
                                "trailing comma not allowed in inline table in TOML v1.0.0",
                            ));
                        }
                        // V1_1: 末尾カンマ許可
                        self.advance_bytes(1);
                        return Ok(Value::Table(result));
                    }
                }
                Some(b'}') => {
                    self.advance_bytes(1);
                    return Ok(Value::Table(result));
                }
                Some(b'\n') | Some(b'\r') => {
                    return Err(Error::parse(
                        self.position(),
                        "newline not allowed in inline table in TOML v1.0.0",
                    ));
                }
                _ => {
                    return Err(self.error("expected ',' or '}' in inline table"));
                }
            }
        }
    }

    /// インラインテーブル内の空白をスキップする。
    ///
    /// v1.0.0: 改行不可（エラー）。
    /// v1.1.0: 空白・コメント・改行をスキップする。
    fn skip_inline_whitespace(&mut self) -> Result<(), Error> {
        if self.version == TomlVersion::V1_1 {
            return self.skip_whitespace_comment_newline();
        }
        while let Some(b) = self.peek() {
            if b == b' ' || b == b'\t' {
                self.advance_bytes(1);
            } else if b == b'\n' || b == b'\r' {
                return Err(Error::parse(
                    self.position(),
                    "newline not allowed in inline table in TOML v1.0.0",
                ));
            } else {
                break;
            }
        }
        Ok(())
    }
}

// ========== ヘルパー関数 ==========

/// パスを文字列表現にする（デバッグ/エラーメッセージ用）。
fn path_to_string(path: &[String]) -> String {
    path.join(".")
}

/// ベアキーに使用できる文字かどうかを返す。
fn is_bare_key_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

/// 制御文字（タブ以外）かどうかを返す。
fn is_control_char(b: u8) -> bool {
    b <= 0x08 || (0x0A..=0x1F).contains(&b) || b == 0x7F
}

/// 複数行基本文字列で禁止される制御文字かどうかを返す。
/// tab (0x09), LF (0x0A), CR (0x0D) は許可。
fn is_ml_basic_control_char(b: u8) -> bool {
    (b <= 0x08) || b == 0x0B || b == 0x0C || (0x0E..=0x1F).contains(&b) || b == 0x7F
}

/// UTF-8 文字の先頭バイトから文字のバイト数を返す。
fn utf8_char_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b < 0xE0 {
        2
    } else if b < 0xF0 {
        3
    } else {
        4
    }
}

/// 10 進数字とアンダースコアの連続をスキャンし、バイト数を返す。
fn scan_digits(s: &str) -> usize {
    s.as_bytes()
        .iter()
        .take_while(|b| b.is_ascii_digit() || **b == b'_')
        .count()
}

/// 16 進数字とアンダースコアの連続をスキャンし、バイト数を返す。
fn scan_hex_digits(s: &str) -> usize {
    s.as_bytes()
        .iter()
        .take_while(|b| b.is_ascii_hexdigit() || **b == b'_')
        .count()
}

/// 8 進数字とアンダースコアの連続をスキャンし、バイト数を返す。
fn scan_oct_digits(s: &str) -> usize {
    s.as_bytes()
        .iter()
        .take_while(|b| (b'0'..=b'7').contains(b) || **b == b'_')
        .count()
}

/// 2 進数字とアンダースコアの連続をスキャンし、バイト数を返す。
fn scan_bin_digits(s: &str) -> usize {
    s.as_bytes()
        .iter()
        .take_while(|b| **b == b'0' || **b == b'1' || **b == b'_')
        .count()
}

/// アンダースコアのルールを検証する。
fn validate_underscore_rules(s: &str, pos: usize) -> Result<(), Error> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return Ok(());
    }
    if bytes[0] == b'_' {
        return Err(Error::parse(
            pos,
            "leading underscore not allowed in number",
        ));
    }
    if bytes[bytes.len() - 1] == b'_' {
        return Err(Error::parse(
            pos,
            "trailing underscore not allowed in number",
        ));
    }
    for window in bytes.windows(2) {
        if window[0] == b'_' && window[1] == b'_' {
            return Err(Error::parse(
                pos,
                "consecutive underscores not allowed in number",
            ));
        }
    }
    Ok(())
}

/// テーブルパスに沿ってルートテーブルからナビゲートし、目的テーブルへの可変参照を返す。
///
/// `Parser::parse_keyval` のフィールド分割借用のためフリー関数として定義。
fn navigate_table_mut<'a>(
    root: &'a mut Table,
    path: &[String],
    array_table_paths: &HashSet<Vec<String>>,
    err_pos: usize,
) -> Result<&'a mut Table, Error> {
    let mut table = root;

    for (i, part) in path.iter().enumerate() {
        let prefix_path = path[..=i].to_vec();

        if array_table_paths.contains(&prefix_path) {
            let entry = table.get_mut(part).ok_or_else(|| {
                Error::parse(
                    err_pos,
                    format!("internal error: array table '{part}' not found"),
                )
            })?;
            let array = entry
                .as_array_mut()
                .ok_or_else(|| Error::parse(err_pos, format!("'{part}' is not an array")))?;
            let last = array
                .last_mut()
                .ok_or_else(|| Error::parse(err_pos, format!("array table '{part}' is empty")))?;
            table = last.as_table_mut().ok_or_else(|| {
                Error::parse(
                    err_pos,
                    format!("element of array table '{part}' is not a table"),
                )
            })?;
        } else {
            let entry = table.get_mut(part).ok_or_else(|| {
                Error::parse(err_pos, format!("internal error: table '{part}' not found"))
            })?;
            table = entry
                .as_table_mut()
                .ok_or_else(|| Error::parse(err_pos, format!("'{part}' is not a table")))?;
        }
    }

    Ok(table)
}

/// dotted key に従って値をテーブルに挿入する。
///
/// `Parser::parse_keyval` のフィールド分割借用のためフリー関数として定義。
fn insert_dotted_key(
    table: &mut Table,
    key_parts: &[String],
    value: Value,
    key_pos: usize,
    current_path: &[String],
    table_states: &mut HashMap<Vec<String>, TableState>,
) -> Result<(), Error> {
    let mut current = table;

    for (i, part) in key_parts[..key_parts.len() - 1].iter().enumerate() {
        let mut full_path = current_path.to_vec();
        full_path.extend(key_parts[..=i].iter().cloned());

        if table_states.get(&full_path) == Some(&TableState::Inline) {
            return Err(Error::parse(
                key_pos,
                format!(
                    "cannot add key to inline table '{}'",
                    path_to_string(&full_path)
                ),
            ));
        }
        if table_states.get(&full_path) == Some(&TableState::Explicit) {
            return Err(Error::parse(
                key_pos,
                format!(
                    "table '{}' is defined by [header] and cannot be extended with dotted key",
                    path_to_string(&full_path)
                ),
            ));
        }

        if current.contains_key(part) {
            match current.get_mut(part) {
                Some(Value::Table(t)) => current = t,
                Some(other) => {
                    return Err(Error::parse(
                        key_pos,
                        format!(
                            "key '{part}' is already defined as {} and cannot be used as a table",
                            other.type_name()
                        ),
                    ));
                }
                None => unreachable!(),
            }
        } else {
            current.insert(part.clone(), Value::Table(Table::new()));
            table_states.entry(full_path).or_insert(TableState::Dotted);
            current = current.get_mut(part).unwrap().as_table_mut().unwrap();
        }
    }

    let final_key = &key_parts[key_parts.len() - 1];

    if current.contains_key(final_key) {
        return Err(Error::parse(
            key_pos,
            format!("key '{final_key}' is already defined"),
        ));
    }

    // インラインテーブルの場合、パスを Inline として登録
    if value.is_table() {
        let mut full_path = current_path.to_vec();
        full_path.extend(key_parts.iter().cloned());
        table_states.insert(full_path, TableState::Inline);
    }

    current.insert(final_key.clone(), value);

    Ok(())
}
