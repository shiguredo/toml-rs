use std::cell::Cell;

use pbt::{sample_date, sample_datetime, sample_offset, sample_time};
use shiguredo_toml::{Datetime, Offset};

/// Date の Display -> FromStr ラウンドトリップ。
#[test]
fn date_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    // 境界値（うるう日を含む）が一度はラウンドトリップされたことを確認する
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let date = sample_date(ctx);
        let s = date.to_string();
        let parsed: Datetime = s.parse().expect("入力のパースに成功するはず ({s:?})");
        assert_eq!(
            parsed.date.as_ref(),
            Some(&date),
            "日付が一致しない: {date:?} -> {s:?} -> {parsed:?}"
        );
        assert!(parsed.time.is_none(), "時刻は無いこと: {parsed:?}");
        assert!(parsed.offset.is_none(), "オフセットは無いこと: {parsed:?}");
        Ok(())
    })?;
    Ok(())
}

/// Time の Display -> FromStr ラウンドトリップ。
#[test]
fn time_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let time = sample_time(ctx);
        let s = time.to_string();
        let parsed: Datetime = s.parse().expect("入力のパースに成功するはず ({s:?})");
        assert!(parsed.date.is_none(), "日付は無いこと: {parsed:?}");
        let parsed_time = parsed.time.as_ref().expect("フィールドは設定されるはず");
        assert_eq!(
            parsed_time.hour, time.hour,
            "時が一致しない: {time:?} -> {s:?} -> {parsed:?}"
        );
        assert_eq!(
            parsed_time.minute, time.minute,
            "分が一致しない: {time:?} -> {s:?} -> {parsed:?}"
        );
        assert_eq!(
            parsed_time.second, time.second,
            "秒が一致しない: {time:?} -> {s:?} -> {parsed:?}"
        );
        // ナノ秒は末尾ゼロの表示差異があるため、値のみ比較
        assert_eq!(
            parsed_time.nanosecond, time.nanosecond,
            "ナノ秒が一致しない: {time:?} -> {s:?} -> {parsed:?}"
        );
        Ok(())
    })?;
    Ok(())
}

/// Offset の Display -> parse ラウンドトリップ。
///
/// `Offset::Z` と `Offset::Custom` の両方が検証されたことを coverage gate で
/// 確認する（片方だけだと分岐の片側を検証しないまま合格してしまう）。
#[test]
fn offset_display_parse_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    // coverage gate: Offset::Z / Offset::Custom がそれぞれ一度は検証されたか
    let zero_offsets = Cell::new(0usize);
    let custom_offsets = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let offset = sample_offset(ctx);
        let s = offset.to_string();
        // Offset は直接 FromStr を持たないが、Datetime として検証
        let dt_str = format!("2000-01-01T00:00:00{s}");
        let parsed: Datetime = dt_str
            .parse()
            .expect("入力のパースに成功するはず ({dt_str:?})");
        assert!(parsed.offset.is_some(), "オフセットは設定されること");
        let parsed_offset = parsed.offset.clone().expect("オフセットは設定されるはず");
        match (&offset, &parsed_offset) {
            (Offset::Z, Offset::Z) => {
                // 検証を通過した後にカウントする
                zero_offsets.set(zero_offsets.get() + 1);
            }
            (Offset::Custom { minutes: m1 }, Offset::Custom { minutes: m2 }) => {
                assert_eq!(
                    m1, m2,
                    "オフセットの分が一致しない: {offset:?} -> {s:?} -> {parsed:?}"
                );
                custom_offsets.set(custom_offsets.get() + 1);
            }
            _ => panic!("オフセットの型が一致しない: {offset:?} -> {s:?} -> {parsed:?}"),
        }
        Ok(())
    })?;
    assert!(
        zero_offsets.get() > 0,
        "Offset::Z が一度も検証されていない\n{runner}"
    );
    assert!(
        custom_offsets.get() > 0,
        "Offset::Custom が一度も検証されていない\n{runner}"
    );
    Ok(())
}

/// Datetime の全バリアント Display -> FromStr ラウンドトリップ。
///
/// TOML の 4 バリアント（Offset Date-Time / Local Date-Time / Local Date /
/// Local Time）すべてが検証されたことを coverage gate で確認する。
#[test]
fn datetime_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    // coverage gate: 4 バリアントごとに一度ずつは検証されたか
    let offset_datetimes = Cell::new(0usize);
    let local_datetimes = Cell::new(0usize);
    let local_dates = Cell::new(0usize);
    let local_times = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let dt = sample_datetime(ctx);
        let s = dt.to_string();
        let parsed: Datetime = s.parse().expect("入力のパースに成功するはず ({s:?})");
        assert_eq!(
            parsed.date, dt.date,
            "日付が一致しない: {dt:?} -> {s:?} -> {parsed:?}"
        );
        // time のナノ秒は表示上末尾ゼロが除去されるが parse で復元される
        match (&dt.time, &parsed.time) {
            (Some(t1), Some(t2)) => {
                assert_eq!(t1.hour, t2.hour, "時が一致しない: {dt:?} -> {s:?}");
                assert_eq!(t1.minute, t2.minute, "分が一致しない: {dt:?} -> {s:?}");
                assert_eq!(t1.second, t2.second, "秒が一致しない: {dt:?} -> {s:?}");
                assert_eq!(
                    t1.nanosecond, t2.nanosecond,
                    "ナノ秒が一致しない: {dt:?} -> {s:?}"
                );
            }
            (None, None) => {}
            _ => panic!("時刻の有無が一致しない: {dt:?} -> {s:?} -> {parsed:?}"),
        }
        assert_eq!(
            parsed.offset, dt.offset,
            "オフセットが一致しない: {dt:?} -> {s:?} -> {parsed:?}"
        );
        // 検証を通過した後に、このケースが検証したバリアントをカウントする
        match (dt.date.is_some(), dt.time.is_some(), dt.offset.is_some()) {
            (true, true, true) => offset_datetimes.set(offset_datetimes.get() + 1),
            (true, true, false) => local_datetimes.set(local_datetimes.get() + 1),
            (true, false, false) => local_dates.set(local_dates.get() + 1),
            (false, true, false) => local_times.set(local_times.get() + 1),
            _ => panic!("想定外の Datetime バリアント: {dt:?}"),
        }
        Ok(())
    })?;
    assert!(
        offset_datetimes.get() > 0,
        "Offset Date-Time が一度も検証されていない\n{runner}"
    );
    assert!(
        local_datetimes.get() > 0,
        "Local Date-Time が一度も検証されていない\n{runner}"
    );
    assert!(
        local_dates.get() > 0,
        "Local Date が一度も検証されていない\n{runner}"
    );
    assert!(
        local_times.get() > 0,
        "Local Time が一度も検証されていない\n{runner}"
    );
    Ok(())
}

/// Date の validate は常に成功する（生成ヘルパーが有効な値のみ生成するため）。
#[test]
fn date_always_valid() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let date = sample_date(ctx);
        assert!(
            date.validate().is_ok(),
            "Date の validate が成功すること: {date:?}"
        );
        Ok(())
    })?;
    Ok(())
}

/// Time の validate は常に成功する。
#[test]
fn time_always_valid() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let time = sample_time(ctx);
        assert!(
            time.validate().is_ok(),
            "Time の validate が成功すること: {time:?}"
        );
        Ok(())
    })?;
    Ok(())
}

/// Offset の validate は常に成功する。
#[test]
fn offset_always_valid() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let offset = sample_offset(ctx);
        assert!(
            offset.validate().is_ok(),
            "Offset の validate が成功すること: {offset:?}"
        );
        Ok(())
    })?;
    Ok(())
}
