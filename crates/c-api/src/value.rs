use std::ffi::{CStr, CString, c_char};

use crate::error::{clear_last_error, set_last_error};
use crate::{TomlError, TomlVersion};

/// TOML 値の型を表す列挙型。
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum TomlValueKind {
    TOML_VALUE_STRING = 0,
    TOML_VALUE_INTEGER = 1,
    TOML_VALUE_FLOAT = 2,
    TOML_VALUE_BOOLEAN = 3,
    TOML_VALUE_DATETIME = 4,
    TOML_VALUE_ARRAY = 5,
    TOML_VALUE_TABLE = 6,
}

/// 不透明な TOML テーブル型。パース結果のルートテーブルを保持する。
pub struct TomlTable {
    inner: shiguredo_toml::Table,
    /// シリアライズ結果のキャッシュ
    serialized_cache: Option<CString>,
    /// キー一覧のキャッシュ
    keys_cache: Option<Vec<CString>>,
    /// datetime 文字列のキャッシュ
    datetime_cache: Vec<CString>,
}

// ------------------------------------------------------------------
// パース
// ------------------------------------------------------------------

/// TOML 文字列をパースして TomlTable を返す。TOML v1.0.0 を使用する。
///
/// 成功時は TomlTable へのポインタを返す。失敗時は null を返す。
/// 返された TomlTable は `toml_table_free` で解放する必要がある。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_parse(input: *const c_char) -> *mut TomlTable {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let input = unsafe { CStr::from_ptr(input) };
    let Ok(input) = input.to_str() else {
        return std::ptr::null_mut();
    };
    match shiguredo_toml::from_str(input) {
        Ok(table) => {
            clear_last_error();
            Box::into_raw(Box::new(TomlTable {
                inner: table,
                serialized_cache: None,
                keys_cache: None,
                datetime_cache: Vec::new(),
            }))
        }
        Err(e) => {
            set_last_error(&e);
            std::ptr::null_mut()
        }
    }
}

/// TOML 文字列を指定バージョンでパースして TomlTable を返す。
///
/// 成功時は TomlTable へのポインタを返す。失敗時は null を返す。
/// 返された TomlTable は `toml_table_free` で解放する必要がある。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_parse_with_version(
    input: *const c_char,
    version: TomlVersion,
) -> *mut TomlTable {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let input = unsafe { CStr::from_ptr(input) };
    let Ok(input) = input.to_str() else {
        return std::ptr::null_mut();
    };
    match shiguredo_toml::from_str_with_version(input, version.into()) {
        Ok(table) => {
            clear_last_error();
            Box::into_raw(Box::new(TomlTable {
                inner: table,
                serialized_cache: None,
                keys_cache: None,
                datetime_cache: Vec::new(),
            }))
        }
        Err(e) => {
            set_last_error(&e);
            std::ptr::null_mut()
        }
    }
}

// ------------------------------------------------------------------
// テーブル操作
// ------------------------------------------------------------------

/// TomlTable を解放する。null ポインタは無視する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_free(table: *mut TomlTable) {
    if !table.is_null() {
        let _ = unsafe { Box::from_raw(table) };
    }
}

/// テーブル内のキー数を返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_len(table: *const TomlTable) -> usize {
    if table.is_null() {
        return 0;
    }
    let table = unsafe { &*table };
    table.inner.len()
}

/// テーブルのキー一覧から指定インデックスのキーを返す。
///
/// 範囲外の場合は null を返す。
/// 返されるポインタは TomlTable が解放されるまで有効。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_key_at(table: *mut TomlTable, index: usize) -> *const c_char {
    if table.is_null() {
        return std::ptr::null();
    }
    let table = unsafe { &mut *table };

    // キーキャッシュを初期化する
    if table.keys_cache.is_none() {
        table.keys_cache = Some(
            table
                .inner
                .keys()
                .filter_map(|k| CString::new(k.as_str()).ok())
                .collect(),
        );
    }

    let keys = table.keys_cache.as_ref().unwrap();
    if index >= keys.len() {
        return std::ptr::null();
    }
    keys[index].as_ptr()
}

/// テーブルにキーが存在するかどうかを返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_contains_key(
    table: *const TomlTable,
    key: *const c_char,
) -> bool {
    if table.is_null() || key.is_null() {
        return false;
    }
    let table = unsafe { &*table };
    let key = unsafe { CStr::from_ptr(key) };
    let Ok(key) = key.to_str() else {
        return false;
    };
    table.inner.contains_key(key)
}

/// テーブルから指定キーの値の型を取得する。
///
/// キーが存在しない場合は TOML_ERROR_KEY_NOT_FOUND を返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_kind(
    table: *const TomlTable,
    key: *const c_char,
    out_kind: *mut TomlValueKind,
) -> TomlError {
    if table.is_null() || key.is_null() || out_kind.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let key = unsafe { CStr::from_ptr(key) };
    let Ok(key) = key.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let Some(value) = table.inner.get(key) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    unsafe {
        *out_kind = value_to_kind(value);
    }
    TomlError::TOML_OK
}

/// テーブルから文字列値を取得する。
///
/// 値が文字列でない場合は TOML_ERROR_TYPE_MISMATCH を返す。
/// 返される文字列は null 終端ではない。out_len でバイト長を取得すること。
/// 返されるポインタは TomlTable が解放されるまで有効。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_string(
    table: *const TomlTable,
    key: *const c_char,
    out_value: *mut *const c_char,
    out_len: *mut usize,
) -> TomlError {
    if table.is_null() || key.is_null() || out_value.is_null() || out_len.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let key = unsafe { CStr::from_ptr(key) };
    let Ok(key) = key.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let Some(value) = table.inner.get(key) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(s) = value.as_str() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    unsafe {
        *out_value = s.as_ptr() as *const c_char;
        *out_len = s.len();
    }
    TomlError::TOML_OK
}

/// テーブルから整数値を取得する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_integer(
    table: *const TomlTable,
    key: *const c_char,
    out_value: *mut i64,
) -> TomlError {
    if table.is_null() || key.is_null() || out_value.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let key = unsafe { CStr::from_ptr(key) };
    let Ok(key) = key.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let Some(value) = table.inner.get(key) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(n) = value.as_integer() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    unsafe {
        *out_value = n;
    }
    TomlError::TOML_OK
}

/// テーブルから浮動小数点値を取得する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_float(
    table: *const TomlTable,
    key: *const c_char,
    out_value: *mut f64,
) -> TomlError {
    if table.is_null() || key.is_null() || out_value.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let key = unsafe { CStr::from_ptr(key) };
    let Ok(key) = key.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let Some(value) = table.inner.get(key) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(f) = value.as_float() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    unsafe {
        *out_value = f;
    }
    TomlError::TOML_OK
}

/// テーブルからブール値を取得する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_bool(
    table: *const TomlTable,
    key: *const c_char,
    out_value: *mut bool,
) -> TomlError {
    if table.is_null() || key.is_null() || out_value.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let key = unsafe { CStr::from_ptr(key) };
    let Ok(key) = key.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let Some(value) = table.inner.get(key) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(b) = value.as_bool() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    unsafe {
        *out_value = b;
    }
    TomlError::TOML_OK
}

/// テーブルから日時の文字列表現を取得する。
///
/// 返される文字列は null 終端。TomlTable が解放されるまで有効。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_datetime(
    table: *mut TomlTable,
    key: *const c_char,
    out_value: *mut *const c_char,
) -> TomlError {
    if table.is_null() || key.is_null() || out_value.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &mut *table };
    let key = unsafe { CStr::from_ptr(key) };
    let Ok(key) = key.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let Some(value) = table.inner.get(key) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(dt) = value.as_datetime() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    let s = format!("{dt}");
    let Ok(c_str) = CString::new(s) else {
        return TomlError::TOML_ERROR_SERIALIZE;
    };
    let ptr = c_str.as_ptr();
    table.datetime_cache.push(c_str);
    unsafe {
        *out_value = ptr;
    }
    TomlError::TOML_OK
}

/// テーブルからサブテーブルのキー数を取得する。
///
/// 値がテーブルでない場合は TOML_ERROR_TYPE_MISMATCH を返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_subtable_len(
    table: *const TomlTable,
    key: *const c_char,
    out_len: *mut usize,
) -> TomlError {
    if table.is_null() || key.is_null() || out_len.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let key = unsafe { CStr::from_ptr(key) };
    let Ok(key) = key.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let Some(value) = table.inner.get(key) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(t) = value.as_table() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    unsafe {
        *out_len = t.len();
    }
    TomlError::TOML_OK
}

/// テーブルから配列の要素数を取得する。
///
/// 値が配列でない場合は TOML_ERROR_TYPE_MISMATCH を返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_array_len(
    table: *const TomlTable,
    key: *const c_char,
    out_len: *mut usize,
) -> TomlError {
    if table.is_null() || key.is_null() || out_len.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let key = unsafe { CStr::from_ptr(key) };
    let Ok(key) = key.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let Some(value) = table.inner.get(key) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(a) = value.as_array() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    unsafe {
        *out_len = a.len();
    }
    TomlError::TOML_OK
}

/// ドット区切りパスで値の型を取得する。
///
/// パスは "servers.alpha.port" のような形式。
/// 配列インデックスは "items[0]" のような形式。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_kind_by_path(
    table: *const TomlTable,
    path: *const c_char,
    out_kind: *mut TomlValueKind,
) -> TomlError {
    if table.is_null() || path.is_null() || out_kind.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let path = unsafe { CStr::from_ptr(path) };
    let Ok(path) = path.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let value = shiguredo_toml::Value::Table(table.inner.clone());
    let segments = match shiguredo_toml::parse_value_path(path) {
        Ok(s) => s,
        Err(_) => return TomlError::TOML_ERROR_PARSE,
    };

    let Some(found) = navigate_path(&value, &segments) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    unsafe {
        *out_kind = value_to_kind(found);
    }
    TomlError::TOML_OK
}

/// ドット区切りパスで文字列値を取得する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_string_by_path(
    table: *const TomlTable,
    path: *const c_char,
    out_value: *mut *const c_char,
    out_len: *mut usize,
) -> TomlError {
    if table.is_null() || path.is_null() || out_value.is_null() || out_len.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let path = unsafe { CStr::from_ptr(path) };
    let Ok(path) = path.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let segments = match shiguredo_toml::parse_value_path(path) {
        Ok(s) => s,
        Err(_) => return TomlError::TOML_ERROR_PARSE,
    };

    let value = shiguredo_toml::Value::Table(table.inner.clone());
    let Some(found) = navigate_path(&value, &segments) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(s) = found.as_str() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    unsafe {
        *out_value = s.as_ptr() as *const c_char;
        *out_len = s.len();
    }
    TomlError::TOML_OK
}

/// ドット区切りパスで整数値を取得する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_integer_by_path(
    table: *const TomlTable,
    path: *const c_char,
    out_value: *mut i64,
) -> TomlError {
    if table.is_null() || path.is_null() || out_value.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let path = unsafe { CStr::from_ptr(path) };
    let Ok(path) = path.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let segments = match shiguredo_toml::parse_value_path(path) {
        Ok(s) => s,
        Err(_) => return TomlError::TOML_ERROR_PARSE,
    };

    let value = shiguredo_toml::Value::Table(table.inner.clone());
    let Some(found) = navigate_path(&value, &segments) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(n) = found.as_integer() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    unsafe {
        *out_value = n;
    }
    TomlError::TOML_OK
}

/// ドット区切りパスで浮動小数点値を取得する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_float_by_path(
    table: *const TomlTable,
    path: *const c_char,
    out_value: *mut f64,
) -> TomlError {
    if table.is_null() || path.is_null() || out_value.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let path = unsafe { CStr::from_ptr(path) };
    let Ok(path) = path.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let segments = match shiguredo_toml::parse_value_path(path) {
        Ok(s) => s,
        Err(_) => return TomlError::TOML_ERROR_PARSE,
    };

    let value = shiguredo_toml::Value::Table(table.inner.clone());
    let Some(found) = navigate_path(&value, &segments) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(f) = found.as_float() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    unsafe {
        *out_value = f;
    }
    TomlError::TOML_OK
}

/// ドット区切りパスでブール値を取得する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_table_get_bool_by_path(
    table: *const TomlTable,
    path: *const c_char,
    out_value: *mut bool,
) -> TomlError {
    if table.is_null() || path.is_null() || out_value.is_null() {
        return TomlError::TOML_ERROR_NULL_POINTER;
    }
    let table = unsafe { &*table };
    let path = unsafe { CStr::from_ptr(path) };
    let Ok(path) = path.to_str() else {
        return TomlError::TOML_ERROR_PARSE;
    };

    let segments = match shiguredo_toml::parse_value_path(path) {
        Ok(s) => s,
        Err(_) => return TomlError::TOML_ERROR_PARSE,
    };

    let value = shiguredo_toml::Value::Table(table.inner.clone());
    let Some(found) = navigate_path(&value, &segments) else {
        return TomlError::TOML_ERROR_KEY_NOT_FOUND;
    };

    let Some(b) = found.as_bool() else {
        return TomlError::TOML_ERROR_TYPE_MISMATCH;
    };

    unsafe {
        *out_value = b;
    }
    TomlError::TOML_OK
}

// ------------------------------------------------------------------
// シリアライズ
// ------------------------------------------------------------------

/// TomlTable を TOML 文字列にシリアライズする。
///
/// 成功時は null 終端の文字列ポインタを返す。失敗時は null を返す。
/// 返されるポインタは TomlTable が解放されるか次のシリアライズ呼び出しまで有効。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_serialize(table: *mut TomlTable) -> *const c_char {
    if table.is_null() {
        return std::ptr::null();
    }
    let table = unsafe { &mut *table };

    let value = shiguredo_toml::Value::Table(table.inner.clone());
    match shiguredo_toml::to_string(&value) {
        Ok(s) => {
            clear_last_error();
            let c_str = CString::new(s).unwrap_or_default();
            let ptr = c_str.as_ptr();
            table.serialized_cache = Some(c_str);
            ptr
        }
        Err(e) => {
            set_last_error(&e);
            std::ptr::null()
        }
    }
}

/// TomlTable を整形済み TOML 文字列にシリアライズする。
///
/// 成功時は null 終端の文字列ポインタを返す。失敗時は null を返す。
/// 返されるポインタは TomlTable が解放されるか次のシリアライズ呼び出しまで有効。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_serialize_pretty(table: *mut TomlTable) -> *const c_char {
    if table.is_null() {
        return std::ptr::null();
    }
    let table = unsafe { &mut *table };

    let value = shiguredo_toml::Value::Table(table.inner.clone());
    match shiguredo_toml::to_string_pretty(&value) {
        Ok(s) => {
            clear_last_error();
            let c_str = CString::new(s).unwrap_or_default();
            let ptr = c_str.as_ptr();
            table.serialized_cache = Some(c_str);
            ptr
        }
        Err(e) => {
            set_last_error(&e);
            std::ptr::null()
        }
    }
}

// ------------------------------------------------------------------
// 内部ヘルパー
// ------------------------------------------------------------------

fn value_to_kind(value: &shiguredo_toml::Value) -> TomlValueKind {
    match value {
        shiguredo_toml::Value::String(_) => TomlValueKind::TOML_VALUE_STRING,
        shiguredo_toml::Value::Integer(_) => TomlValueKind::TOML_VALUE_INTEGER,
        shiguredo_toml::Value::Float(_) => TomlValueKind::TOML_VALUE_FLOAT,
        shiguredo_toml::Value::Boolean(_) => TomlValueKind::TOML_VALUE_BOOLEAN,
        shiguredo_toml::Value::Datetime(_) => TomlValueKind::TOML_VALUE_DATETIME,
        shiguredo_toml::Value::Array(_) => TomlValueKind::TOML_VALUE_ARRAY,
        shiguredo_toml::Value::Table(_) => TomlValueKind::TOML_VALUE_TABLE,
    }
}

/// パスセグメントに従って Value を辿る。
fn navigate_path<'a>(
    value: &'a shiguredo_toml::Value,
    segments: &[shiguredo_toml::PathSegment],
) -> Option<&'a shiguredo_toml::Value> {
    let mut current = value;
    for segment in segments {
        match segment {
            shiguredo_toml::PathSegment::Key(key) => {
                current = current.as_table()?.get(key.as_str())?;
            }
            shiguredo_toml::PathSegment::Index(idx) => {
                current = current.as_array()?.get(*idx)?;
            }
        }
    }
    Some(current)
}
