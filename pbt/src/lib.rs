//! PBT 用の値生成ヘルパー。
//!
//! `noprop` の `sample_*` 関数を組み合わせて、TOML の各要素を生成する。
//! シードとケース数は各テスト側で明示する。ケース数は既存の proptest デフォルトと同じ
//! 256 を全テストで統一して使う。

use noprop::TestCaseContext;
use shiguredo_toml::{Date, Datetime, Offset, Table, Time, Value};

/// ベアキーとして有効な文字列を生成する。
pub fn sample_bare_key(ctx: &mut TestCaseContext) -> String {
    // ベアキーに使える文字（英数字とアンダースコアとハイフン）だけを選ぶ
    const CHARS: [u8; 64] = *b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_-";
    let len = noprop::sample_usize_in(ctx, 1..=16);
    let mut key = String::new();
    for _ in 0..len {
        key.push(noprop::sample_choice(ctx, &CHARS) as char);
    }
    key
}

/// TOML の日付を生成する。
pub fn sample_date(ctx: &mut TestCaseContext) -> Date {
    // 日数の上限は月ごとに異なり、2 月はうるう年で変わる
    let year = noprop::sample_usize_in(ctx, 1900..=2100) as u16;
    let month = noprop::sample_usize_in(ctx, 1..=12) as u8;
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31u8,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) {
                29
            } else {
                28
            }
        }
        _ => unreachable!(),
    };
    let day = noprop::sample_usize_in(ctx, 1..=max_day as usize) as u8;
    Date { year, month, day }
}

/// TOML の時刻を生成する。
pub fn sample_time(ctx: &mut TestCaseContext) -> Time {
    Time {
        hour: noprop::sample_usize_in(ctx, 0..=23) as u8,
        minute: noprop::sample_usize_in(ctx, 0..=59) as u8,
        second: noprop::sample_usize_in(ctx, 0..=59) as u8,
        nanosecond: noprop::sample_usize_in(ctx, 0..1_000_000_000) as u32,
    }
}

/// TOML のオフセットを生成する。
pub fn sample_offset(ctx: &mut TestCaseContext) -> Offset {
    if noprop::sample_bool(ctx) {
        Offset::Z
    } else {
        // オフセットの分は -1439 から 1439 まで（RFC 3339 の time-numoffset は ±23:59
        // まで。refs/v1.0.0.md の日時節は RFC 3339 に委譲している）
        Offset::Custom {
            minutes: noprop::sample_usize_in(ctx, 0..=2878) as i16 - 1439,
        }
    }
}

/// TOML の Datetime を生成する（4 バリアントすべて）。
pub fn sample_datetime(ctx: &mut TestCaseContext) -> Datetime {
    // TOML の日時は Offset Date-Time / Local Date-Time / Local Date / Local Time の
    // 4 バリアントがあり、それぞれ等確率で選ぶ
    match noprop::sample_weighted_index(ctx, &[1, 1, 1, 1]) {
        0 => Datetime {
            date: Some(sample_date(ctx)),
            time: Some(sample_time(ctx)),
            offset: Some(sample_offset(ctx)),
        },
        1 => Datetime {
            date: Some(sample_date(ctx)),
            time: Some(sample_time(ctx)),
            offset: None,
        },
        2 => Datetime {
            date: Some(sample_date(ctx)),
            time: None,
            offset: None,
        },
        _ => Datetime {
            date: None,
            time: Some(sample_time(ctx)),
            offset: None,
        },
    }
}

/// TOML の基本文字列として安全な文字列を生成する。
///
/// 制御文字を除外し、エスケープ可能な文字のみを含む。
pub fn sample_safe_string(ctx: &mut TestCaseContext) -> String {
    let len = noprop::sample_usize_in(ctx, 0..=63);
    let mut s = String::new();
    for _ in 0..len {
        // ASCII 印字可能文字と一部の非 ASCII Unicode 文字を等確率で選ぶ
        match noprop::sample_weighted_index(ctx, &[1, 1]) {
            0 => s.push(noprop::sample_ascii_printable_char(ctx)),
            _ => {
                // 非 ASCII 文字はアクセント付きラテン文字・日本語・絵文字から選ぶ
                s.push(match noprop::sample_weighted_index(ctx, &[1, 1, 1]) {
                    0 => '\u{00E9}',  // é（アクセント付きラテン文字）
                    1 => '\u{3042}',  // あ（日本語）
                    _ => '\u{1F600}', // U+1F600（絵文字。有効な Unicode）
                });
            }
        }
    }
    s
}

/// リーフ（非再帰）の TOML 値を生成する。
fn sample_leaf_value(ctx: &mut TestCaseContext) -> Value {
    // NaN と Infinity は比較できないため、浮動小数点数は有限の範囲に絞る
    match noprop::sample_weighted_index(ctx, &[1, 1, 1, 1, 1]) {
        0 => Value::String(sample_safe_string(ctx)),
        1 => Value::Integer(noprop::sample_i64(ctx)),
        2 => Value::Float(noprop::sample_f64_in(ctx, -1e100, 1e100)),
        3 => Value::Boolean(noprop::sample_bool(ctx)),
        _ => Value::Datetime(sample_datetime(ctx)),
    }
}

/// TOML の Value を再帰的に生成する（深さ制限付き）。
pub fn sample_value(ctx: &mut TestCaseContext) -> Value {
    sample_value_recursive(ctx, 0)
}

/// 深さを明示しながら Value を再帰的に生成する。
fn sample_value_recursive(ctx: &mut TestCaseContext, depth: usize) -> Value {
    // 深さ 3 を超えたらリーフだけを生成する（配列とテーブルの無限再帰を防ぐ）
    if depth >= 3 {
        return sample_leaf_value(ctx);
    }
    // リーフを 4 割、配列とテーブルを各 3 割の割合で選ぶ
    match noprop::sample_weighted_index(ctx, &[4, 3, 3]) {
        0 => sample_leaf_value(ctx),
        1 => {
            let len = noprop::sample_usize_in(ctx, 0..=4);
            let mut array = Vec::new();
            for _ in 0..len {
                array.push(sample_value_recursive(ctx, depth + 1));
            }
            Value::Array(array)
        }
        _ => {
            let len = noprop::sample_usize_in(ctx, 0..=4);
            let mut table = Table::new();
            for _ in 0..len {
                table.insert(sample_bare_key(ctx), sample_value_recursive(ctx, depth + 1));
            }
            Value::Table(table)
        }
    }
}

/// ルートテーブルとして有効な Table を生成する。
pub fn sample_table(ctx: &mut TestCaseContext) -> Table {
    let len = noprop::sample_usize_in(ctx, 0..=8);
    let mut table = Table::new();
    for _ in 0..len {
        table.insert(sample_bare_key(ctx), sample_value(ctx));
    }
    table
}
