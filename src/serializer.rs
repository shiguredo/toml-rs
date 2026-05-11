use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};

use crate::error::Error;
use crate::value::{Table, Value};

/// シリアライザの最大再帰深度。パーサーと同じ値。
const MAX_DEPTH: usize = 128;

/// Value を TOML 文字列に変換する。
pub(crate) fn to_string(value: &Value) -> Result<String, Error> {
    let mut ser = Serializer::new(false);
    ser.serialize_value(value, &[])?;
    Ok(ser.output)
}

/// Value を単一値の TOML 表現に変換する。
pub(crate) fn to_inline_string(value: &Value) -> Result<String, Error> {
    let mut ser = Serializer::new(false);
    ser.write_inline_value(value)?;
    Ok(ser.output)
}

/// Value を整形済み TOML 文字列に変換する。
pub(crate) fn to_string_pretty(value: &Value) -> Result<String, Error> {
    let mut ser = Serializer::new(true);
    ser.serialize_value(value, &[])?;
    Ok(ser.output)
}

/// TOML シリアライザ。
struct Serializer {
    /// 出力バッファ
    output: String,
    /// 整形モード（テーブル間に空行を挿入する）
    pretty: bool,
    /// 現在の再帰深度
    depth: usize,
}

impl Serializer {
    fn new(pretty: bool) -> Self {
        Self {
            output: String::new(),
            pretty,
            depth: 0,
        }
    }

    /// 再帰深度を 1 増やし、制限を超えていればエラーを返す。
    fn enter_depth(&mut self) -> Result<(), Error> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(Error::serialize(format!(
                "nesting depth exceeds maximum of {MAX_DEPTH}"
            )));
        }
        Ok(())
    }

    /// 再帰深度を 1 減らす。
    fn leave_depth(&mut self) {
        self.depth -= 1;
    }

    /// トップレベルの値を直列化する。
    fn serialize_value(&mut self, value: &Value, path: &[String]) -> Result<(), Error> {
        match value {
            Value::Table(table) => self.serialize_table(table, path),
            _ => Err(Error::serialize("top-level value must be a table")),
        }
    }

    /// テーブルを直列化する。
    ///
    /// 手順:
    /// 1. 単純キー値ペア（非テーブル、非配列テーブル）を出力
    /// 2. サブテーブルを `[header]` セクションとして出力
    /// 3. 配列テーブルを `[[header]]` セクションとして出力
    fn serialize_table(&mut self, table: &Table, path: &[String]) -> Result<(), Error> {
        self.enter_depth()?;

        // 1. 単純キー値ペア
        for (key, value) in table {
            if is_inline_value(value) {
                self.write_key(key);
                self.output.push_str(" = ");
                self.write_inline_value(value)?;
                self.output.push('\n');
            }
        }

        // 2. サブテーブル（テーブル型の値で、配列テーブルではないもの）
        for (key, value) in table {
            if let Value::Table(sub_table) = value {
                let mut sub_path = path.to_vec();
                sub_path.push(key.clone());

                if self.pretty && !self.output.is_empty() && !self.output.ends_with("\n\n") {
                    self.output.push('\n');
                }

                self.output.push('[');
                self.write_path(&sub_path);
                self.output.push_str("]\n");

                self.serialize_table(sub_table, &sub_path)?;
            }
        }

        // 3. 配列テーブル（テーブルの配列）
        for (key, value) in table {
            if let Value::Array(array) = value
                && is_array_of_tables(array)
            {
                let mut sub_path = path.to_vec();
                sub_path.push(key.clone());

                for element in array {
                    if let Value::Table(sub_table) = element {
                        if self.pretty && !self.output.is_empty() && !self.output.ends_with("\n\n")
                        {
                            self.output.push('\n');
                        }

                        self.output.push_str("[[");
                        self.write_path(&sub_path);
                        self.output.push_str("]]\n");

                        self.serialize_table(sub_table, &sub_path)?;
                    }
                }
            }
        }

        self.leave_depth();
        Ok(())
    }

    /// キーを出力する（必要に応じてクォート）。
    fn write_key(&mut self, key: &str) {
        if needs_quoting(key) {
            self.write_quoted_key(key);
        } else {
            self.output.push_str(key);
        }
    }

    /// クォート付きキーを出力する。
    fn write_quoted_key(&mut self, key: &str) {
        self.output.push('"');
        write_escaped_string(&mut self.output, key);
        self.output.push('"');
    }

    /// ドット区切りパスを出力する。
    fn write_path(&mut self, path: &[String]) {
        for (i, part) in path.iter().enumerate() {
            if i > 0 {
                self.output.push('.');
            }
            self.write_key(part);
        }
    }

    /// インライン値（テーブル以外、または配列テーブル以外の配列）を出力する。
    fn write_inline_value(&mut self, value: &Value) -> Result<(), Error> {
        match value {
            Value::String(s) => {
                self.output.push('"');
                write_escaped_string(&mut self.output, s);
                self.output.push('"');
            }
            Value::Integer(n) => {
                self.output.push_str(&n.to_string());
            }
            Value::Float(f) => {
                self.write_float(*f);
            }
            Value::Boolean(b) => {
                self.output.push_str(if *b { "true" } else { "false" });
            }
            Value::Datetime(dt) => {
                self.output.push_str(&dt.to_string());
            }
            Value::Array(arr) => {
                self.write_inline_array(arr)?;
            }
            Value::Table(table) => {
                self.write_inline_table(table)?;
            }
        }
        Ok(())
    }

    /// 浮動小数点数を出力する。
    fn write_float(&mut self, f: f64) {
        if f.is_nan() {
            self.output.push_str("nan");
        } else if f.is_infinite() {
            if f.is_sign_positive() {
                self.output.push_str("inf");
            } else {
                self.output.push_str("-inf");
            }
        } else {
            let s = format!("{f}");
            self.output.push_str(&s);
            // 整数っぽい表記になった場合は .0 を追加
            if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                self.output.push_str(".0");
            }
        }
    }

    /// インライン配列を出力する。
    fn write_inline_array(&mut self, array: &[Value]) -> Result<(), Error> {
        self.enter_depth()?;
        self.output.push('[');
        for (i, value) in array.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.write_inline_value(value)?;
        }
        self.output.push(']');
        self.leave_depth();
        Ok(())
    }

    /// インラインテーブルを出力する。
    fn write_inline_table(&mut self, table: &Table) -> Result<(), Error> {
        self.enter_depth()?;
        self.output.push('{');
        for (i, (key, value)) in table.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.write_key(key);
            self.output.push_str(" = ");
            self.write_inline_value(value)?;
        }
        self.output.push('}');
        self.leave_depth();
        Ok(())
    }
}

/// 値がインライン出力可能か（テーブルでも配列テーブルでもない）を返す。
fn is_inline_value(value: &Value) -> bool {
    match value {
        Value::Table(_) => false,
        Value::Array(arr) => !is_array_of_tables(arr),
        _ => true,
    }
}

/// 配列がテーブルの配列かどうかを返す。
///
/// 空配列はテーブルの配列ではない。全要素がテーブルの場合のみ true。
fn is_array_of_tables(array: &[Value]) -> bool {
    !array.is_empty() && array.iter().all(|v| v.is_table())
}

/// キーにクォートが必要かどうかを返す。
pub(crate) fn needs_quoting(key: &str) -> bool {
    key.is_empty()
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// TOML キーを適切にフォーマットする（必要に応じてクォートする）。
pub(crate) fn format_key(key: &str) -> String {
    if needs_quoting(key) {
        let mut out = String::with_capacity(key.len() + 2);
        out.push('"');
        write_escaped_string(&mut out, key);
        out.push('"');
        out
    } else {
        key.to_owned()
    }
}

/// エスケープ済み文字列を出力バッファに書き込む（クォートなし）。
pub(crate) fn write_escaped_string(output: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '\u{0008}' => output.push_str("\\b"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\u{000C}' => output.push_str("\\f"),
            '\r' => output.push_str("\\r"),
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            c if c.is_control() => {
                let code = c as u32;
                if code <= 0xFFFF {
                    output.push_str(&format!("\\u{code:04X}"));
                } else {
                    output.push_str(&format!("\\U{code:08X}"));
                }
            }
            c => output.push(c),
        }
    }
}
