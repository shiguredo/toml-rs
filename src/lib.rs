//! TOML v1.0.0 / v1.1.0 ライブラリ。
//!
//! 外部依存なしで TOML v1.0.0 および v1.1.0 仕様に完全準拠するパーサとシリアライザを提供する。

mod datetime;
mod edit;
mod error;
mod parser;
mod serializer;
mod span;
mod value;

pub use datetime::{Date, Datetime, Offset, Time};
pub use edit::Document;
pub use error::Error;
pub use span::{
    CommentIndex, CommentSpan, PathSegment, SpanIndex, TextSpan, ValuePath, parse_value_path,
};
pub use value::{Array, Table, Value};

/// TOML バージョン。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TomlVersion {
    /// TOML v1.0.0
    V1_0,
    /// TOML v1.1.0
    V1_1,
}

/// TOML 文字列を解析して Table に変換する。
///
/// TOML ドキュメントのルートは常にテーブルである。デフォルトは TOML v1.0.0。
pub fn from_str(s: &str) -> Result<Table, Error> {
    parser::parse(s, TomlVersion::V1_0)
}

/// TOML 文字列を指定バージョンで解析して Table に変換する。
pub fn from_str_with_version(s: &str, version: TomlVersion) -> Result<Table, Error> {
    parser::parse(s, version)
}

/// Value を TOML 文字列に変換する。
pub fn to_string(value: &Value) -> Result<String, Error> {
    serializer::to_string(value)
}

/// 単一 Value を TOML の単一値テキストに変換する。
pub fn to_inline_string(value: &Value) -> Result<String, Error> {
    serializer::to_inline_string(value)
}

/// Value を整形済み TOML 文字列に変換する。
pub fn to_string_pretty(value: &Value) -> Result<String, Error> {
    serializer::to_string_pretty(value)
}
