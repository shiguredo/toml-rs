use std::collections::BTreeMap;
use std::fmt;
use std::ops::Index;
use std::str::FromStr;

use crate::Error;
use crate::datetime::Datetime;

/// TOML テーブル型。キー順でソートされる。
pub type Table = BTreeMap<String, Value>;

/// TOML 配列型。
pub type Array = Vec<Value>;

/// TOML 値。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// TOML 文字列
    String(String),
    /// TOML 整数 (64 bit 符号付き)
    Integer(i64),
    /// TOML 浮動小数点数 (IEEE 754 binary64)
    Float(f64),
    /// TOML ブール値
    Boolean(bool),
    /// TOML 日時
    Datetime(Datetime),
    /// TOML 配列
    Array(Array),
    /// TOML テーブル
    Table(Table),
}

impl Value {
    /// 値が文字列かどうかを返す。
    pub fn is_str(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// 値が整数かどうかを返す。
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    /// 値が浮動小数点数かどうかを返す。
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// 値がブール値かどうかを返す。
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// 値が日時かどうかを返す。
    pub fn is_datetime(&self) -> bool {
        matches!(self, Value::Datetime(_))
    }

    /// 値が配列かどうかを返す。
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// 値がテーブルかどうかを返す。
    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table(_))
    }

    /// 文字列への参照を返す。文字列でない場合は `None`。
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// 整数値を返す。整数でない場合は `None`。
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// 浮動小数点値を返す。浮動小数点でない場合は `None`。
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// ブール値を返す。ブールでない場合は `None`。
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// 日時への参照を返す。日時でない場合は `None`。
    pub fn as_datetime(&self) -> Option<&Datetime> {
        match self {
            Value::Datetime(dt) => Some(dt),
            _ => None,
        }
    }

    /// 配列への参照を返す。配列でない場合は `None`。
    pub fn as_array(&self) -> Option<&Array> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// 配列への可変参照を返す。配列でない場合は `None`。
    pub fn as_array_mut(&mut self) -> Option<&mut Array> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// テーブルへの参照を返す。テーブルでない場合は `None`。
    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Value::Table(t) => Some(t),
            _ => None,
        }
    }

    /// テーブルへの可変参照を返す。テーブルでない場合は `None`。
    pub fn as_table_mut(&mut self) -> Option<&mut Table> {
        match self {
            Value::Table(t) => Some(t),
            _ => None,
        }
    }

    /// 値の型名を返す。
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "string",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Boolean(_) => "boolean",
            Value::Datetime(_) => "datetime",
            Value::Array(_) => "array",
            Value::Table(_) => "table",
        }
    }

    /// テーブルからキーで値を取得する。テーブルでない場合は `None`。
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_table()?.get(key)
    }

    /// テーブルからキーで値の可変参照を取得する。テーブルでない場合は `None`。
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.as_table_mut()?.get_mut(key)
    }
}

/// テーブルからキーで値を取得する。
///
/// # Panics
///
/// - 値がテーブルでない場合
/// - 指定キーがテーブルに存在しない場合
impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, key: &str) -> &Value {
        self.get(key)
            .unwrap_or_else(|| panic!("key '{key}' not found"))
    }
}

/// 配列からインデックスで値を取得する。
///
/// # Panics
///
/// - 値が配列でない場合
/// - インデックスが配列の範囲外の場合
impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Value {
        match self {
            Value::Array(arr) => &arr[index],
            _ => panic!("index access on non-array value"),
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_owned())
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Integer(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Integer(i64::from(n))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<Datetime> for Value {
    fn from(dt: Datetime) -> Self {
        Value::Datetime(dt)
    }
}

impl From<Array> for Value {
    fn from(arr: Array) -> Self {
        Value::Array(arr)
    }
}

impl From<Table> for Value {
    fn from(t: Table) -> Self {
        Value::Table(t)
    }
}

impl FromStr for Value {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::from_str(s).map(Value::Table)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match crate::to_string(self) {
            Ok(s) => f.write_str(&s),
            Err(_) => Err(fmt::Error),
        }
    }
}
