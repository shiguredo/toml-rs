use std::fmt;
use std::num::NonZeroUsize;

/// TOML の解析・直列化エラー。
#[derive(Debug)]
pub enum Error {
    /// 解析エラー（バイト位置付き）
    Parse {
        /// エラーメッセージ
        message: String,
        /// 入力中のバイト位置
        position: usize,
    },
    /// 直列化エラー
    Serialize {
        /// エラーメッセージ
        message: String,
    },
}

impl Error {
    /// 解析エラーの場合、入力中のバイト位置を返す。
    pub fn position(&self) -> Option<usize> {
        match self {
            Error::Parse { position, .. } => Some(*position),
            Error::Serialize { .. } => None,
        }
    }

    /// 解析エラーの場合、バイト位置から行番号と列番号を算出する。
    ///
    /// 行番号・列番号は 1 始まり。
    pub fn get_line_and_column(&self, text: &str) -> Option<(NonZeroUsize, NonZeroUsize)> {
        let position = self.position()?;
        let position = position.min(text.len());

        let mut line = 1usize;
        let mut col = 1usize;

        for (i, ch) in text.char_indices() {
            if i >= position {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        Some((NonZeroUsize::new(line)?, NonZeroUsize::new(col)?))
    }

    /// 解析エラーの場合、エラーが発生した行のテキストを返す。
    pub fn get_line<'a>(&self, text: &'a str) -> Option<&'a str> {
        let position = self.position()?;
        let position = position.min(text.len());

        let start = text[..position].rfind('\n').map_or(0, |i| i + 1);
        let end = text[position..]
            .find('\n')
            .map_or(text.len(), |i| position + i);

        Some(&text[start..end])
    }

    pub(crate) fn parse(position: usize, message: impl Into<String>) -> Self {
        Error::Parse {
            message: message.into(),
            position,
        }
    }

    pub(crate) fn serialize(message: impl Into<String>) -> Self {
        Error::Serialize {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse { message, position } => {
                write!(f, "parse error at byte {position}: {message}")
            }
            Error::Serialize { message } => {
                write!(f, "serialize error: {message}")
            }
        }
    }
}

impl std::error::Error for Error {}
