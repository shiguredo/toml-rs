use pbt::{date_strategy, datetime_strategy, offset_strategy, time_strategy};
use proptest::prelude::*;
use shiguredo_toml::{Datetime, Offset};

proptest! {
    /// Date の Display -> FromStr ラウンドトリップ。
    #[test]
    fn date_roundtrip(date in date_strategy()) {
        let s = date.to_string();
        let parsed: Datetime = s.parse().unwrap();
        prop_assert_eq!(parsed.date.as_ref(), Some(&date));
        prop_assert!(parsed.time.is_none());
        prop_assert!(parsed.offset.is_none());
    }

    /// Time の Display -> FromStr ラウンドトリップ。
    #[test]
    fn time_roundtrip(time in time_strategy()) {
        let s = time.to_string();
        let parsed: Datetime = s.parse().unwrap();
        prop_assert!(parsed.date.is_none());
        let parsed_time = parsed.time.as_ref().unwrap();
        prop_assert_eq!(parsed_time.hour, time.hour);
        prop_assert_eq!(parsed_time.minute, time.minute);
        prop_assert_eq!(parsed_time.second, time.second);
        // ナノ秒は末尾ゼロの表示差異があるため、値のみ比較
        prop_assert_eq!(parsed_time.nanosecond, time.nanosecond);
    }

    /// Offset の Display -> parse ラウンドトリップ。
    #[test]
    fn offset_display_parse_roundtrip(offset in offset_strategy()) {
        let s = offset.to_string();
        // Offset は直接 FromStr を持たないが、Datetime として検証
        let dt_str = format!("2000-01-01T00:00:00{s}");
        let parsed: Datetime = dt_str.parse().unwrap();
        prop_assert!(parsed.offset.is_some());
        let parsed_offset = parsed.offset.unwrap();
        match (&offset, &parsed_offset) {
            (Offset::Z, Offset::Z) => {}
            (Offset::Custom { minutes: m1 }, Offset::Custom { minutes: m2 }) => {
                prop_assert_eq!(m1, m2);
            }
            _ => prop_assert!(false, "Offset type mismatch"),
        }
    }

    /// Datetime の全バリアント Display -> FromStr ラウンドトリップ。
    #[test]
    fn datetime_roundtrip(dt in datetime_strategy()) {
        let s = dt.to_string();
        if s.is_empty() {
            // date=None, time=None の場合はスキップ
            return Ok(());
        }
        let parsed: Datetime = s.parse().unwrap();
        prop_assert_eq!(parsed.date, dt.date);
        // time のナノ秒は表示上末尾ゼロが除去されるが parse で復元される
        match (&dt.time, &parsed.time) {
            (Some(t1), Some(t2)) => {
                prop_assert_eq!(t1.hour, t2.hour);
                prop_assert_eq!(t1.minute, t2.minute);
                prop_assert_eq!(t1.second, t2.second);
                prop_assert_eq!(t1.nanosecond, t2.nanosecond);
            }
            (None, None) => {}
            _ => prop_assert!(false, "Time presence mismatch"),
        }
        prop_assert_eq!(parsed.offset, dt.offset);
    }

    /// Date の validate は常に成功する（Strategy が有効な値のみ生成するため）。
    #[test]
    fn date_always_valid(date in date_strategy()) {
        prop_assert!(date.validate().is_ok());
    }

    /// Time の validate は常に成功する。
    #[test]
    fn time_always_valid(time in time_strategy()) {
        prop_assert!(time.validate().is_ok());
    }

    /// Offset の validate は常に成功する。
    #[test]
    fn offset_always_valid(offset in offset_strategy()) {
        prop_assert!(offset.validate().is_ok());
    }
}
