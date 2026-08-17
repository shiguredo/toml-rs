use pbt::{sample_date, sample_datetime, sample_offset, sample_time};
use shiguredo_toml::{Datetime, Offset};

/// Date の Display -> FromStr ラウンドトリップ。
#[test]
fn date_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let date = sample_date(ctx);
        let s = date.to_string();
        let parsed: Datetime = s.parse().expect("入力のパースに成功するはず");
        assert_eq!(parsed.date.as_ref(), Some(&date), "日付が一致すること");
        assert!(parsed.time.is_none(), "時刻は無いこと");
        assert!(parsed.offset.is_none(), "オフセットは無いこと");
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
        let parsed: Datetime = s.parse().expect("入力のパースに成功するはず");
        assert!(parsed.date.is_none(), "日付は無いこと");
        let parsed_time = parsed.time.as_ref().expect("フィールドは設定されるはず");
        assert_eq!(parsed_time.hour, time.hour, "時が一致すること");
        assert_eq!(parsed_time.minute, time.minute, "分が一致すること");
        assert_eq!(parsed_time.second, time.second, "秒が一致すること");
        // ナノ秒は末尾ゼロの表示差異があるため、値のみ比較
        assert_eq!(
            parsed_time.nanosecond, time.nanosecond,
            "ナノ秒が一致すること"
        );
        Ok(())
    })?;
    Ok(())
}

/// Offset の Display -> parse ラウンドトリップ。
#[test]
fn offset_display_parse_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let offset = sample_offset(ctx);
        let s = offset.to_string();
        // Offset は直接 FromStr を持たないが、Datetime として検証
        let dt_str = format!("2000-01-01T00:00:00{s}");
        let parsed: Datetime = dt_str.parse().expect("入力のパースに成功するはず");
        assert!(parsed.offset.is_some(), "オフセットは設定されること");
        let parsed_offset = parsed.offset.expect("オフセットは設定されるはず");
        match (&offset, &parsed_offset) {
            (Offset::Z, Offset::Z) => {}
            (Offset::Custom { minutes: m1 }, Offset::Custom { minutes: m2 }) => {
                assert_eq!(m1, m2, "オフセットの分が一致すること");
            }
            _ => panic!("オフセットの型が一致しない"),
        }
        Ok(())
    })?;
    Ok(())
}

/// Datetime の全バリアント Display -> FromStr ラウンドトリップ。
#[test]
fn datetime_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let dt = sample_datetime(ctx);
        let s = dt.to_string();
        let parsed: Datetime = s.parse().expect("入力のパースに成功するはず");
        assert_eq!(parsed.date, dt.date, "日付が一致すること");
        // time のナノ秒は表示上末尾ゼロが除去されるが parse で復元される
        match (&dt.time, &parsed.time) {
            (Some(t1), Some(t2)) => {
                assert_eq!(t1.hour, t2.hour, "時が一致すること");
                assert_eq!(t1.minute, t2.minute, "分が一致すること");
                assert_eq!(t1.second, t2.second, "秒が一致すること");
                assert_eq!(t1.nanosecond, t2.nanosecond, "ナノ秒が一致すること");
            }
            (None, None) => {}
            _ => panic!("時刻の有無が一致しない"),
        }
        assert_eq!(parsed.offset, dt.offset, "オフセットが一致すること");
        Ok(())
    })?;
    Ok(())
}

/// Date の validate は常に成功する（生成ヘルパーが有効な値のみ生成するため）。
#[test]
fn date_always_valid() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("NOPROP_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(256, |ctx| {
        let date = sample_date(ctx);
        assert!(date.validate().is_ok(), "Date の validate が成功すること");
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
        assert!(time.validate().is_ok(), "Time の validate が成功すること");
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
            "Offset の validate が成功すること"
        );
        Ok(())
    })?;
    Ok(())
}
