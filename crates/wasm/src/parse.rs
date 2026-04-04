//! TOML パース・シリアライズの wasm 向け関数群

use std::ffi::{CStr, c_char};

/// TOML 文字列をパースし、正規化した TOML 文字列として返す。TOML v1.0.0 を使用する。
///
/// # 引数
///
/// - `input`: null 終端の TOML 文字列
///
/// # 戻り値
///
/// TOML 文字列を含む `Vec<u8>` へのポインタ。エラー時は NULL。
///
/// 呼び出し元は不要になったら `toml_wasm_vec_free` で解放すること。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_wasm_parse(input: *const c_char) -> *mut Vec<u8> {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let input = unsafe { CStr::from_ptr(input) };
    let Ok(input) = input.to_str() else {
        return std::ptr::null_mut();
    };

    let table = match shiguredo_toml::from_str(input) {
        Ok(t) => t,
        Err(_) => return std::ptr::null_mut(),
    };

    let value = shiguredo_toml::Value::Table(table);
    match shiguredo_toml::to_string(&value) {
        Ok(s) => {
            let bytes: Vec<u8> = s.into_bytes();
            Box::into_raw(Box::new(bytes))
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// TOML 文字列を指定バージョンでパースし、正規化した TOML 文字列として返す。
///
/// # 引数
///
/// - `input`: null 終端の TOML 文字列
/// - `version`: TOML バージョン (0 = v1.0.0, 1 = v1.1.0)
///
/// # 戻り値
///
/// TOML 文字列を含む `Vec<u8>` へのポインタ。エラー時は NULL。
///
/// 呼び出し元は不要になったら `toml_wasm_vec_free` で解放すること。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_wasm_parse_with_version(
    input: *const c_char,
    version: u32,
) -> *mut Vec<u8> {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let input = unsafe { CStr::from_ptr(input) };
    let Ok(input) = input.to_str() else {
        return std::ptr::null_mut();
    };

    let version = match version {
        0 => shiguredo_toml::TomlVersion::V1_0,
        1 => shiguredo_toml::TomlVersion::V1_1,
        _ => return std::ptr::null_mut(),
    };

    let table = match shiguredo_toml::from_str_with_version(input, version) {
        Ok(t) => t,
        Err(_) => return std::ptr::null_mut(),
    };

    let value = shiguredo_toml::Value::Table(table);
    match shiguredo_toml::to_string(&value) {
        Ok(s) => {
            let bytes: Vec<u8> = s.into_bytes();
            Box::into_raw(Box::new(bytes))
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// TOML 文字列をパースし、整形済み TOML 文字列として返す。
///
/// # 引数
///
/// - `input`: null 終端の TOML 文字列
///
/// # 戻り値
///
/// 整形済み TOML 文字列を含む `Vec<u8>` へのポインタ。エラー時は NULL。
///
/// 呼び出し元は不要になったら `toml_wasm_vec_free` で解放すること。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_wasm_serialize_pretty(input: *const c_char) -> *mut Vec<u8> {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let input = unsafe { CStr::from_ptr(input) };
    let Ok(input) = input.to_str() else {
        return std::ptr::null_mut();
    };

    let table = match shiguredo_toml::from_str(input) {
        Ok(t) => t,
        Err(_) => return std::ptr::null_mut(),
    };

    let value = shiguredo_toml::Value::Table(table);
    match shiguredo_toml::to_string_pretty(&value) {
        Ok(s) => {
            let bytes: Vec<u8> = s.into_bytes();
            Box::into_raw(Box::new(bytes))
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// TOML 文字列のバリデーションを行う。
///
/// # 引数
///
/// - `input`: null 終端の TOML 文字列
/// - `version`: TOML バージョン (0 = v1.0.0, 1 = v1.1.0)
///
/// # 戻り値
///
/// 有効な TOML の場合は 1、無効な場合は 0 を返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn toml_wasm_validate(input: *const c_char, version: u32) -> u32 {
    if input.is_null() {
        return 0;
    }
    let input = unsafe { CStr::from_ptr(input) };
    let Ok(input) = input.to_str() else {
        return 0;
    };

    let version = match version {
        0 => shiguredo_toml::TomlVersion::V1_0,
        1 => shiguredo_toml::TomlVersion::V1_1,
        _ => return 0,
    };

    match shiguredo_toml::from_str_with_version(input, version) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}
