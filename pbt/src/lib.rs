//! PBT 用の Strategy ヘルパー。

use proptest::prelude::*;
use shiguredo_toml::{Date, Datetime, Offset, Table, Time, Value};

/// ベアキーとして有効な文字列を生成する Strategy。
pub fn bare_key_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9_-]{1,16}"
}

/// TOML の日付を生成する Strategy。
pub fn date_strategy() -> impl Strategy<Value = Date> {
    (1900u16..2100, 1u8..=12u8).prop_flat_map(|(year, month)| {
        let max_day = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31u8,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    29
                } else {
                    28
                }
            }
            _ => unreachable!(),
        };
        (Just(year), Just(month), 1u8..=max_day).prop_map(|(y, m, d)| Date {
            year: y,
            month: m,
            day: d,
        })
    })
}

/// TOML の時刻を生成する Strategy。
pub fn time_strategy() -> impl Strategy<Value = Time> {
    (0u8..=23, 0u8..=59, 0u8..=59, 0u32..1_000_000_000).prop_map(|(h, m, s, ns)| Time {
        hour: h,
        minute: m,
        second: s,
        nanosecond: ns,
    })
}

/// TOML のオフセットを生成する Strategy。
pub fn offset_strategy() -> impl Strategy<Value = Offset> {
    prop_oneof![
        Just(Offset::Z),
        (-1439i16..=1439i16).prop_map(|minutes| Offset::Custom { minutes }),
    ]
}

/// TOML の Datetime を生成する Strategy（全バリアント）。
pub fn datetime_strategy() -> impl Strategy<Value = Datetime> {
    prop_oneof![
        // Offset Date-Time
        (date_strategy(), time_strategy(), offset_strategy()).prop_map(|(d, t, o)| Datetime {
            date: Some(d),
            time: Some(t),
            offset: Some(o),
        }),
        // Local Date-Time
        (date_strategy(), time_strategy()).prop_map(|(d, t)| Datetime {
            date: Some(d),
            time: Some(t),
            offset: None,
        }),
        // Local Date
        date_strategy().prop_map(|d| Datetime {
            date: Some(d),
            time: None,
            offset: None,
        }),
        // Local Time
        time_strategy().prop_map(|t| Datetime {
            date: None,
            time: Some(t),
            offset: None,
        }),
    ]
}

/// TOML の基本文字列として安全な文字列を生成する Strategy。
///
/// 制御文字を除外し、エスケープ可能な文字のみを含む。
pub fn safe_string_strategy() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            // ASCII 印字可能文字（バックスラッシュとクォートを含む）
            (0x20u32..=0x7E).prop_map(|c| {
                char::from_u32(c)
                    .expect("char::from_u32() must succeed for ASCII printable characters")
            }),
            // 一部の非 ASCII Unicode 文字
            prop_oneof![
                Just('\u{00E9}'),  // é（アクセント付きラテン文字）
                Just('\u{3042}'),  // あ（日本語）
                Just('\u{1F600}'), // 😀（絵文字。有効な Unicode）
            ],
        ],
        0..64,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

/// リーフ（非再帰）の TOML 値を生成する Strategy。
fn leaf_value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        safe_string_strategy().prop_map(Value::String),
        any::<i64>().prop_map(Value::Integer),
        // NaN と Infinity を除外（比較不可能のため）
        (-1e100f64..1e100).prop_map(Value::Float),
        any::<bool>().prop_map(Value::Boolean),
        datetime_strategy().prop_map(Value::Datetime),
    ]
}

/// TOML の Value を再帰的に生成する Strategy（深さ制限付き）。
pub fn value_strategy() -> impl Strategy<Value = Value> {
    leaf_value_strategy().prop_recursive(
        3,  // 最大深さ
        64, // 最大ノード数
        8,  // 各レベルの最大要素数
        |inner| {
            prop_oneof![
                // 配列
                proptest::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
                // テーブル
                proptest::collection::btree_map(bare_key_strategy(), inner, 0..4)
                    .prop_map(Value::Table),
            ]
        },
    )
}

/// ルートテーブルとして有効な Table を生成する Strategy。
pub fn table_strategy() -> impl Strategy<Value = Table> {
    proptest::collection::btree_map(bare_key_strategy(), value_strategy(), 0..8)
}
