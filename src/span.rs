use std::collections::HashMap;

/// 値が元テキスト上に存在していたバイト範囲。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSpan {
    /// 開始バイト位置（含む）。
    pub start: usize,
    /// 終了バイト位置（含まない）。
    pub end: usize,
}

impl TextSpan {
    /// 新しい範囲を作成する。
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// 値パスの 1 セグメント。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    /// テーブルキー。
    Key(String),
    /// 配列インデックス。
    Index(usize),
}

/// ルートから値へのパス。
pub type ValuePath = Vec<PathSegment>;

/// 値パスとテキスト範囲のインデックス。
#[derive(Debug, Clone, Default)]
pub struct SpanIndex {
    spans: HashMap<ValuePath, TextSpan>,
}

impl SpanIndex {
    /// 空のインデックスを作成する。
    pub fn new() -> Self {
        Self::default()
    }

    /// パスに対応する範囲を取得する。
    pub fn get(&self, path: &[PathSegment]) -> Option<TextSpan> {
        self.spans.get(path).copied()
    }

    /// すべてのエントリを列挙する。
    pub fn iter(&self) -> impl Iterator<Item = (&ValuePath, &TextSpan)> {
        self.spans.iter()
    }

    pub(crate) fn insert(&mut self, path: ValuePath, span: TextSpan) {
        self.spans.insert(path, span);
    }
}

/// コメント位置情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentSpan {
    /// コメントそのものの範囲（`#` を含む）。
    pub span: TextSpan,
    /// 行末コメントの場合の紐づけ先値パス。
    pub target: Option<ValuePath>,
}

/// コメントのインデックス。
#[derive(Debug, Clone, Default)]
pub struct CommentIndex {
    comments: Vec<CommentSpan>,
    trailing_comments: HashMap<ValuePath, TextSpan>,
}

impl CommentIndex {
    /// 空のインデックスを作成する。
    pub fn new() -> Self {
        Self::default()
    }

    /// すべてのコメントエントリを列挙する。
    pub fn iter(&self) -> impl Iterator<Item = &CommentSpan> {
        self.comments.iter()
    }

    /// 指定値パスに紐づく行末コメント範囲を取得する。
    pub fn trailing_for(&self, path: &[PathSegment]) -> Option<TextSpan> {
        self.trailing_comments.get(path).copied()
    }

    pub(crate) fn insert(&mut self, span: TextSpan, target: Option<ValuePath>) {
        if let Some(path) = target.as_ref() {
            self.trailing_comments.insert(path.clone(), span);
        }
        self.comments.push(CommentSpan { span, target });
    }
}

/// 文字列パスを `ValuePath` に変換する。
///
/// 例: `servers[1].port`
pub fn parse_value_path(path: &str) -> Result<ValuePath, String> {
    if path.is_empty() {
        return Err("value path is empty".to_owned());
    }

    let bytes = path.as_bytes();
    let mut i = 0usize;
    let mut segments = Vec::new();

    while i < bytes.len() {
        if bytes[i] == b'.' {
            return Err("value path contains empty key".to_owned());
        }

        if bytes[i] == b'[' {
            let (index, next_i) = parse_index(path, i)?;
            segments.push(PathSegment::Index(index));
            i = next_i;
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b'.' && bytes[i] != b'[' {
                i += 1;
            }
            if start == i {
                return Err("value path key is empty".to_owned());
            }
            segments.push(PathSegment::Key(path[start..i].to_owned()));
        }

        while i < bytes.len() && bytes[i] == b'[' {
            let (index, next_i) = parse_index(path, i)?;
            segments.push(PathSegment::Index(index));
            i = next_i;
        }

        if i < bytes.len() {
            if bytes[i] != b'.' {
                return Err(format!("invalid value path syntax: '{}'", path));
            }
            i += 1;
            if i >= bytes.len() {
                return Err("value path ends with '.'".to_owned());
            }
        }
    }

    Ok(segments)
}

fn parse_index(path: &str, start: usize) -> Result<(usize, usize), String> {
    let bytes = path.as_bytes();
    if bytes.get(start) != Some(&b'[') {
        return Err("invalid array index start".to_owned());
    }

    let mut i = start + 1;
    let num_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if num_start == i {
        return Err("array index is empty".to_owned());
    }
    if bytes.get(i) != Some(&b']') {
        return Err("array index missing closing ']'".to_owned());
    }

    let index = path[num_start..i]
        .parse::<usize>()
        .map_err(|e| format!("array index conversion error: {e}"))?;
    Ok((index, i + 1))
}
