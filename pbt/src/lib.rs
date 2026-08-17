//! PBT 用の値生成ヘルパー。
//!
//! `noprop` の `sample_*` 関数を組み合わせて、TOML の各要素を生成する。
//! シードとケース数は各テスト側で明示する。ケース数は既存の proptest デフォルトと同じ
//! 256 を全テストで統一して使う。
//!
//! 一様生成だけでは出にくい境界値（月末、うるう日、オフセット上限、長さの上限、
//! エスケープが必要な文字など）を `sample_with_boundaries` と `sample_ratio` で
//! 一定の割合で混ぜ込み、境界を確実に検証に入れる。

use noprop::TestCaseContext;
use shiguredo_toml::{Date, Datetime, Offset, Table, Time, Value};

/// ベアキーとして有効な文字列を生成する。
///
/// 長さの両端（1 文字と 16 文字）を確実に含める。
#[track_caller]
pub fn sample_bare_key(ctx: &mut TestCaseContext) -> String {
    // ベアキーに使える文字（英数字とアンダースコアとハイフン）だけを選ぶ
    const CHARS: [u8; 64] = *b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_-";
    // 長さ 1 と 16 を 4 分の 1 の確率で混ぜ、残りは一様に選ぶ
    let len =
        noprop::sample_with_boundaries(ctx, &[1usize, 16], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 1..=16)
        });
    let mut key = String::new();
    for _ in 0..len {
        key.push(noprop::sample_choice(ctx, &CHARS) as char);
    }
    key
}

/// TOML の日付を生成する。
///
/// うるう日（2 月 29 日）と月末（28 / 29 / 30 / 31 日）を確実に含める。
#[track_caller]
pub fn sample_date(ctx: &mut TestCaseContext) -> Date {
    // うるう日は通常の一様生成ではほぼ現れないため、8 分の 1 の確率で
    // うるう年の固定値（2000-02-29）を返す。うるう年判定の分岐を確実に検証する。
    if noprop::sample_ratio(ctx, noprop::Ratio::one_nth(8)) {
        return Date {
            year: 2000,
            month: 2,
            day: 29,
        };
    }
    // 年の両端と 2000 年（うるう年）を境界に含める
    let year = noprop::sample_with_boundaries(
        ctx,
        &[1900usize, 2000, 2100],
        noprop::Ratio::one_nth(4),
        |ctx| noprop::sample_usize_in(ctx, 1900..=2100),
    ) as u16;
    // 2 月と両端の月を境界に含める
    let month =
        noprop::sample_with_boundaries(ctx, &[1usize, 2, 12], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 1..=12)
        }) as u8;
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) {
                29
            } else {
                28
            }
        }
        _ => unreachable!("month は 1..=12 に絞っている"),
    };
    // 1 日と月末を境界に含める
    let day =
        noprop::sample_with_boundaries(ctx, &[1usize, max_day], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 1..=max_day)
        }) as u8;
    Date { year, month, day }
}

/// TOML の時刻を生成する。
///
/// 各フィールドの下限と上限を境界に含める。
#[track_caller]
pub fn sample_time(ctx: &mut TestCaseContext) -> Time {
    // 時（0 と 23）、分（0 と 59）、秒（0 と 59）を境界に含める
    let hour =
        noprop::sample_with_boundaries(ctx, &[0usize, 23], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 0..=23)
        }) as u8;
    let minute =
        noprop::sample_with_boundaries(ctx, &[0usize, 59], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 0..=59)
        }) as u8;
    let second =
        noprop::sample_with_boundaries(ctx, &[0usize, 59], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 0..=59)
        }) as u8;
    // ナノ秒の下限と上限（末尾ゼロの trim が正しく復元されるかも検証する）
    let nanosecond = noprop::sample_with_boundaries(
        ctx,
        &[0usize, 999_999_999],
        noprop::Ratio::one_nth(4),
        |ctx| noprop::sample_usize_in(ctx, 0..1_000_000_000),
    ) as u32;
    Time {
        hour,
        minute,
        second,
        nanosecond,
    }
}

/// TOML のオフセットを生成する。
///
/// オフセットの分は -1439 から 1439 まで（RFC 3339 の time-numoffset は ±23:59
/// まで。refs/v1.0.0.md の日時節は RFC 3339 に委譲している）。上限を境界に含める。
#[track_caller]
pub fn sample_offset(ctx: &mut TestCaseContext) -> Offset {
    if noprop::sample_bool(ctx) {
        Offset::Z
    } else {
        // 分の上限（±1439 分）を境界に含める
        let minutes =
            noprop::sample_with_boundaries(ctx, &[0usize, 2878], noprop::Ratio::one_nth(2), |ctx| {
                noprop::sample_usize_in(ctx, 0..=2878)
            }) as i16
                - 1439;
        Offset::Custom { minutes }
    }
}

/// TOML の Datetime を生成する（4 バリアントすべて）。
///
/// TOML の日時は Offset Date-Time / Local Date-Time / Local Date / Local Time の
/// 4 バリアントがあり、それぞれ等確率で選ぶ。
#[track_caller]
pub fn sample_datetime(ctx: &mut TestCaseContext) -> Datetime {
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
/// 制御文字を除外し、エスケープ可能な文字のみを含む。長さの上限（63 文字）を
/// 境界に含め、エスケープが必要な文字を確実に出現させる。
#[track_caller]
pub fn sample_safe_string(ctx: &mut TestCaseContext) -> String {
    // 長さの両端（0 と 63）を 4 分の 1 の確率で混ぜ、残りは一様に選ぶ
    let len =
        noprop::sample_with_boundaries(ctx, &[0usize, 63], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 0..=63)
        });
    let mut s = String::new();
    for _ in 0..len {
        s.push(sample_string_char(ctx));
    }
    s
}

/// 基本文字列に現れ得る 1 文字を生成する。
///
/// エスケープが必要な文字（バックスラッシュとダブルクォート）と非 ASCII 文字を
/// 一定の割合で混ぜ、シリアライザのエスケープが正しく再パースされることを
/// 確実に検証する。
#[track_caller]
fn sample_string_char(ctx: &mut TestCaseContext) -> char {
    match noprop::sample_weighted_index(ctx, &[4, 3, 3]) {
        // 通常の ASCII 印字可能文字（スペースからチルダまで）
        0 => noprop::sample_ascii_printable_char(ctx),
        // エスケープが必要な文字
        1 => match noprop::sample_weighted_index(ctx, &[1, 1]) {
            0 => '\\',
            _ => '"',
        },
        // 非 ASCII 文字はアクセント付きラテン文字・日本語・絵文字から選ぶ
        _ => match noprop::sample_weighted_index(ctx, &[1, 1, 1]) {
            0 => '\u{00E9}',  // é（アクセント付きラテン文字）
            1 => '\u{3042}',  // あ（日本語）
            _ => '\u{1F600}', // U+1F600（絵文字。有効な Unicode）
        },
    }
}

/// リーフ（非再帰）の TOML 値を生成する。
///
/// NaN と Infinity は比較できないため、浮動小数点数は有限の範囲に絞る。
#[track_caller]
fn sample_leaf_value(ctx: &mut TestCaseContext) -> Value {
    match noprop::sample_weighted_index(ctx, &[1, 1, 1, 1, 1]) {
        0 => Value::String(sample_safe_string(ctx)),
        1 => Value::Integer(sample_i64_boundaries(ctx)),
        2 => Value::Float(sample_f64_boundaries(ctx)),
        3 => Value::Boolean(noprop::sample_bool(ctx)),
        _ => Value::Datetime(sample_datetime(ctx)),
    }
}

/// 整数の生成に境界値（最小・最大・ゼロ）を含める。
#[track_caller]
fn sample_i64_boundaries(ctx: &mut TestCaseContext) -> i64 {
    noprop::sample_with_boundaries(
        ctx,
        &[i64::MIN, i64::MAX, 0],
        noprop::Ratio::one_nth(2),
        |ctx| noprop::sample_i64(ctx),
    )
}

/// 有限浮動小数点数の生成に境界値（両端とゼロ）を含める。
#[track_caller]
pub fn sample_f64_boundaries(ctx: &mut TestCaseContext) -> f64 {
    noprop::sample_with_boundaries(
        ctx,
        &[-1e100, 0.0, 1e100],
        noprop::Ratio::one_nth(2),
        |ctx| noprop::sample_f64_in(ctx, -1e100, 1e100),
    )
}

/// TOML の Value を再帰的に生成する（深さ制限付き）。
#[track_caller]
pub fn sample_value(ctx: &mut TestCaseContext) -> Value {
    sample_value_recursive(ctx, 0)
}

/// 深さを明示しながら Value を再帰的に生成する。
///
/// 深さ 3 を超えたらリーフだけを生成する（配列とテーブルの無限再帰を防ぐ）。
/// リーフを 4 割、配列とテーブルを各 3 割の割合で選ぶ。
#[track_caller]
fn sample_value_recursive(ctx: &mut TestCaseContext, depth: usize) -> Value {
    if depth >= 3 {
        return sample_leaf_value(ctx);
    }
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
///
/// 空テーブルと上限サイズ（8 要素）を確実に含める。
#[track_caller]
pub fn sample_table(ctx: &mut TestCaseContext) -> Table {
    // 要素数の両端（0 と 8）を 4 分の 1 の確率で混ぜ、残りは一様に選ぶ
    let len = noprop::sample_with_boundaries(ctx, &[0usize, 8], noprop::Ratio::one_nth(4), |ctx| {
        noprop::sample_usize_in(ctx, 0..=8)
    });
    let mut table = Table::new();
    for _ in 0..len {
        table.insert(sample_bare_key(ctx), sample_value(ctx));
    }
    table
}
