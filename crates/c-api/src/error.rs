use std::ffi::{CString, c_char};

/// TOML 操作のエラーコード。
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum TomlError {
    /// 成功
    TOML_OK = 0,
    /// 解析エラー
    TOML_ERROR_PARSE,
    /// 直列化エラー
    TOML_ERROR_SERIALIZE,
    /// バリデーションエラー
    TOML_ERROR_VALIDATE,
    /// null ポインタが渡された
    TOML_ERROR_NULL_POINTER,
    /// 型の不一致
    TOML_ERROR_TYPE_MISMATCH,
    /// キーが見つからない
    TOML_ERROR_KEY_NOT_FOUND,
    /// インデックスが範囲外
    TOML_ERROR_INDEX_OUT_OF_RANGE,
}

impl From<&shiguredo_toml::Error> for TomlError {
    fn from(e: &shiguredo_toml::Error) -> Self {
        match e {
            shiguredo_toml::Error::Parse { .. } => TomlError::TOML_ERROR_PARSE,
            shiguredo_toml::Error::Serialize { .. } => TomlError::TOML_ERROR_SERIALIZE,
            shiguredo_toml::Error::Validate { .. } => TomlError::TOML_ERROR_VALIDATE,
        }
    }
}

/// 最後のエラー情報を保持する構造体。
///
/// スレッドローカルに保持し、エラー発生時に更新する。
pub(crate) struct LastError {
    pub message: Option<CString>,
    pub position: Option<usize>,
}

thread_local! {
    static LAST_ERROR: std::cell::RefCell<LastError> = const { std::cell::RefCell::new(LastError {
        message: None,
        position: None,
    })};
}

/// エラー情報を記録する。
pub(crate) fn set_last_error(error: &shiguredo_toml::Error) {
    let message = CString::new(error.to_string()).unwrap_or_default();
    let position = error.position();
    LAST_ERROR.with(|e| {
        let mut e = e.borrow_mut();
        e.message = Some(message);
        e.position = position;
    });
}

/// エラー情報をクリアする。
pub(crate) fn clear_last_error() {
    LAST_ERROR.with(|e| {
        let mut e = e.borrow_mut();
        e.message = None;
        e.position = None;
    });
}

/// 最後のエラーメッセージを返す。
///
/// エラーがない場合は空文字列のポインタを返す。
/// 返されるポインタは次のエラー発生まで有効。
#[unsafe(no_mangle)]
pub extern "C" fn toml_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        let e = e.borrow();
        match &e.message {
            Some(msg) => msg.as_ptr(),
            None => c"".as_ptr(),
        }
    })
}

/// 最後の解析エラーのバイト位置を返す。
///
/// 解析エラーでない場合は -1 を返す。
#[unsafe(no_mangle)]
pub extern "C" fn toml_get_last_error_position() -> i64 {
    LAST_ERROR.with(|e| {
        let e = e.borrow();
        match e.position {
            Some(pos) => pos as i64,
            None => -1,
        }
    })
}
