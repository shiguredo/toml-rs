use std::fmt;
use std::str::FromStr;

use crate::Error;

/// TOML の日付部分 (YYYY-MM-DD)。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Date {
    /// 年 (0-9999)
    pub year: u16,
    /// 月 (1-12)
    pub month: u8,
    /// 日 (1-28/29/30/31)
    pub day: u8,
}

/// TOML の時刻部分 (HH:MM:SS.nanosecond)。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Time {
    /// 時 (0-23)
    pub hour: u8,
    /// 分 (0-59)
    pub minute: u8,
    /// 秒 (0-59)
    pub second: u8,
    /// ナノ秒 (0-999_999_999)
    pub nanosecond: u32,
}

/// TOML のオフセット部分。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Offset {
    /// UTC (Z)
    Z,
    /// カスタムオフセット (分単位、例: +09:00 = 540)
    Custom {
        /// オフセットの分数 (-1439..=1439)
        minutes: i16,
    },
}

/// TOML の日時型。
///
/// 4 つの TOML 日時バリアントをすべて表現する:
///
/// - Offset Date-Time: `date=Some`, `time=Some`, `offset=Some`
/// - Local Date-Time: `date=Some`, `time=Some`, `offset=None`
/// - Local Date: `date=Some`, `time=None`, `offset=None`
/// - Local Time: `date=None`, `time=Some`, `offset=None`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Datetime {
    /// 日付部分
    pub date: Option<Date>,
    /// 時刻部分
    pub time: Option<Time>,
    /// オフセット部分
    pub offset: Option<Offset>,
}

/// 閏年かどうかを判定する。
fn is_leap_year(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// 指定された年月の最大日数を返す。
fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

impl Date {
    /// 日付を検証する。
    pub fn validate(&self) -> Result<(), Error> {
        if self.month < 1 || self.month > 12 {
            return Err(Error::validate(format!(
                "month out of range: {}",
                self.month
            )));
        }
        let max_day = days_in_month(self.year, self.month);
        if self.day < 1 || self.day > max_day {
            return Err(Error::validate(format!(
                "day out of range: max {} for {}-{:02}, got {}",
                max_day, self.year, self.month, self.day
            )));
        }
        Ok(())
    }
}

impl Time {
    /// 時刻を検証する。
    pub fn validate(&self) -> Result<(), Error> {
        if self.hour > 23 {
            return Err(Error::validate(format!("hour out of range: {}", self.hour)));
        }
        if self.minute > 59 {
            return Err(Error::validate(format!(
                "minute out of range: {}",
                self.minute
            )));
        }
        if self.second > 59 {
            return Err(Error::validate(format!(
                "second out of range: {}",
                self.second
            )));
        }
        if self.nanosecond > 999_999_999 {
            return Err(Error::validate(format!(
                "nanosecond out of range: {}",
                self.nanosecond
            )));
        }
        Ok(())
    }
}

impl Offset {
    /// オフセットを検証する。
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Offset::Z => Ok(()),
            Offset::Custom { minutes } => {
                if *minutes < -1439 || *minutes > 1439 {
                    return Err(Error::validate(format!(
                        "offset out of range: {} minutes",
                        minutes
                    )));
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)?;
        if self.nanosecond > 0 {
            // ナノ秒を表示し、末尾のゼロを削除する
            let s = format!("{:09}", self.nanosecond);
            let trimmed = s.trim_end_matches('0');
            write!(f, ".{trimmed}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Offset::Z => write!(f, "Z"),
            Offset::Custom { minutes } => {
                let sign = if *minutes >= 0 { '+' } else { '-' };
                let abs = minutes.unsigned_abs();
                let h = abs / 60;
                let m = abs % 60;
                write!(f, "{sign}{h:02}:{m:02}")
            }
        }
    }
}

impl fmt::Display for Datetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.date, &self.time, &self.offset) {
            (Some(d), Some(t), Some(o)) => write!(f, "{d}T{t}{o}"),
            (Some(d), Some(t), None) => write!(f, "{d}T{t}"),
            (Some(d), None, _) => write!(f, "{d}"),
            (None, Some(t), _) => write!(f, "{t}"),
            (None, None, _) => Ok(()),
        }
    }
}

/// 入力からちょうど `n` 桁の数値を読み取る。
fn parse_n_digits(s: &str, n: usize) -> Result<(u32, &str), Error> {
    if s.len() < n {
        return Err(Error::validate(format!(
            "expected {n}-digit number but input is too short"
        )));
    }
    let digits = &s[..n];
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::validate(format!(
            "expected {n}-digit number but found '{digits}'"
        )));
    }
    let value = digits
        .parse::<u32>()
        .map_err(|e| Error::validate(format!("number conversion error: {e}")))?;
    Ok((value, &s[n..]))
}

/// 入力の先頭が指定バイトかチェックし、消費する。
fn expect_byte(s: &str, expected: u8) -> Result<&str, Error> {
    match s.as_bytes().first() {
        Some(&b) if b == expected => Ok(&s[1..]),
        Some(&b) => Err(Error::validate(format!(
            "expected '{}' but found '{}'",
            expected as char, b as char
        ))),
        None => Err(Error::validate(format!(
            "expected '{}' but input ended",
            expected as char
        ))),
    }
}

impl FromStr for Datetime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_datetime_str(s)
    }
}

/// 日時文字列を解析する。内部ヘルパー。TOML v1.0.0 として解析する。
pub(crate) fn parse_datetime_str(s: &str) -> Result<Datetime, Error> {
    parse_datetime_str_with_version(s, crate::TomlVersion::V1_0)
}

/// 日時文字列を指定バージョンで解析する。内部ヘルパー。
pub(crate) fn parse_datetime_str_with_version(
    s: &str,
    version: crate::TomlVersion,
) -> Result<Datetime, Error> {
    // 時刻のみ (HH:MM[:SS]...)
    if s.len() >= 2
        && s.as_bytes()[0].is_ascii_digit()
        && s.as_bytes()[1].is_ascii_digit()
        && s.as_bytes().get(2) == Some(&b':')
    {
        // ただし YYYY-MM-DD の場合を除外（4 桁目が '-' の場合は日付）
        if s.len() >= 5 && s.as_bytes()[4] == b'-' {
            // 日付の可能性がある
        } else {
            let (time, rest) = parse_time_part_with_version(s, version)?;
            if !rest.is_empty() {
                return Err(Error::validate(format!(
                    "unexpected characters after time: '{rest}'"
                )));
            }
            return Ok(Datetime {
                date: None,
                time: Some(time),
                offset: None,
            });
        }
    }

    // 日付始まり (YYYY-MM-DD...)
    let (date, rest) = parse_date_part(s)?;

    if rest.is_empty() {
        // Local Date のみ
        return Ok(Datetime {
            date: Some(date),
            time: None,
            offset: None,
        });
    }

    // T または空白デリミタ
    let rest = match rest.as_bytes().first() {
        Some(&b'T') | Some(&b't') => &rest[1..],
        Some(&b' ') => &rest[1..],
        Some(&b) => {
            return Err(Error::validate(format!(
                "expected 'T' or space after date but found '{}'",
                b as char
            )));
        }
        None => unreachable!(),
    };

    let (time, rest) = parse_time_part_with_version(rest, version)?;

    // オフセット
    let (offset, rest) = if rest.is_empty() {
        (None, rest)
    } else {
        match rest.as_bytes().first() {
            Some(&b'Z') | Some(&b'z') => (Some(Offset::Z), &rest[1..]),
            Some(&b'+') | Some(&b'-') => {
                let (offset, remaining) = parse_offset_part(rest)?;
                (Some(offset), remaining)
            }
            _ => (None, rest),
        }
    };

    if !rest.is_empty() {
        return Err(Error::validate(format!(
            "unexpected characters after datetime: '{rest}'"
        )));
    }

    Ok(Datetime {
        date: Some(date),
        time: Some(time),
        offset,
    })
}

fn parse_date_part(s: &str) -> Result<(Date, &str), Error> {
    let (year, rest) = parse_n_digits(s, 4)?;
    let rest = expect_byte(rest, b'-')?;
    let (month, rest) = parse_n_digits(rest, 2)?;
    let rest = expect_byte(rest, b'-')?;
    let (day, rest) = parse_n_digits(rest, 2)?;

    let date = Date {
        year: year as u16,
        month: month as u8,
        day: day as u8,
    };
    date.validate()?;

    Ok((date, rest))
}

fn parse_time_part_with_version(
    s: &str,
    version: crate::TomlVersion,
) -> Result<(Time, &str), Error> {
    let (hour, rest) = parse_n_digits(s, 2)?;
    let rest = expect_byte(rest, b':')?;
    let (minute, rest) = parse_n_digits(rest, 2)?;

    // V1_1: 秒は省略可能。次が ':' でなければ秒を 0 補完する。
    let (second, rest) = if version == crate::TomlVersion::V1_1 && !rest.starts_with(':') {
        (0u32, rest)
    } else {
        let rest = expect_byte(rest, b':')?;
        parse_n_digits(rest, 2)?
    };

    let (nanosecond, rest) = if rest.as_bytes().first() == Some(&b'.') {
        let rest = &rest[1..];
        // 小数秒部分: 1 桁以上必要
        let digit_count = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
        if digit_count == 0 {
            return Err(Error::validate(
                "fractional seconds require at least one digit",
            ));
        }
        let digits = &rest[..digit_count];
        // 9 桁にパディングまたは切り捨て
        let nanosecond = if digit_count <= 9 {
            let mut padded = String::from(digits);
            while padded.len() < 9 {
                padded.push('0');
            }
            padded
                .parse::<u32>()
                .map_err(|e| Error::validate(format!("nanosecond conversion error: {e}")))?
        } else {
            // 9 桁を超える場合は切り捨て（四捨五入禁止）
            rest[..9]
                .parse::<u32>()
                .map_err(|e| Error::validate(format!("nanosecond conversion error: {e}")))?
        };
        (nanosecond, &rest[digit_count..])
    } else {
        (0, rest)
    };

    let time = Time {
        hour: hour as u8,
        minute: minute as u8,
        second: second as u8,
        nanosecond,
    };
    time.validate()?;

    Ok((time, rest))
}

fn parse_offset_part(s: &str) -> Result<(Offset, &str), Error> {
    let sign = s.as_bytes()[0];
    let rest = &s[1..];
    let (hours, rest) = parse_n_digits(rest, 2)?;
    let rest = expect_byte(rest, b':')?;
    let (minutes, rest) = parse_n_digits(rest, 2)?;

    if minutes > 59 {
        return Err(Error::validate(format!(
            "offset minutes out of range: {minutes}"
        )));
    }

    let total_minutes = (hours * 60 + minutes) as i16;
    let total_minutes = if sign == b'-' {
        -total_minutes
    } else {
        total_minutes
    };

    let offset = Offset::Custom {
        minutes: total_minutes,
    };
    offset.validate()?;

    Ok((offset, rest))
}
