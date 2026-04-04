#![allow(clippy::missing_safety_doc)]

mod error;
mod value;
mod version;

pub use error::*;
pub use value::*;
pub use version::*;

use std::ffi::c_char;

/// ライブラリバージョン文字列を返す。
///
/// 返されるポインタは静的領域を指し、解放不要。
#[unsafe(no_mangle)]
pub extern "C" fn toml_library_version() -> *const c_char {
    c"SHIGUREDO_TOML_VERSION".as_ptr()
}
