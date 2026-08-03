use shiguredo_toml::{Date, Datetime, Offset, Time};

mod parse_error {
    use super::*;

    #[test]
    fn invalid_month_zero() {
        let dt = Date {
            year: 2024,
            month: 0,
            day: 1,
        };
        assert!(dt.validate().is_err());
    }

    #[test]
    fn invalid_month_13() {
        let dt = Date {
            year: 2024,
            month: 13,
            day: 1,
        };
        assert!(dt.validate().is_err());
    }

    #[test]
    fn invalid_day_zero() {
        let dt = Date {
            year: 2024,
            month: 1,
            day: 0,
        };
        assert!(dt.validate().is_err());
    }

    #[test]
    fn invalid_day_32_jan() {
        let dt = Date {
            year: 2024,
            month: 1,
            day: 32,
        };
        assert!(dt.validate().is_err());
    }

    #[test]
    fn feb_29_non_leap_year() {
        let dt = Date {
            year: 2023,
            month: 2,
            day: 29,
        };
        assert!(dt.validate().is_err());
    }

    #[test]
    fn feb_29_leap_year() {
        let dt = Date {
            year: 2024,
            month: 2,
            day: 29,
        };
        assert!(dt.validate().is_ok());
    }

    #[test]
    fn feb_29_century_non_leap() {
        let dt = Date {
            year: 1900,
            month: 2,
            day: 29,
        };
        assert!(dt.validate().is_err());
    }

    #[test]
    fn feb_29_400_year_leap() {
        let dt = Date {
            year: 2000,
            month: 2,
            day: 29,
        };
        assert!(dt.validate().is_ok());
    }

    #[test]
    fn invalid_hour_24() {
        let t = Time {
            hour: 24,
            minute: 0,
            second: 0,
            nanosecond: 0,
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn invalid_minute_60() {
        let t = Time {
            hour: 0,
            minute: 60,
            second: 0,
            nanosecond: 0,
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn invalid_second_60() {
        let t = Time {
            hour: 0,
            minute: 0,
            second: 60,
            nanosecond: 0,
        };
        assert!(t.validate().is_err());
    }

    #[test]
    fn offset_out_of_range() {
        let o = Offset::Custom { minutes: 1440 };
        assert!(o.validate().is_err());
    }

    #[test]
    fn offset_negative_out_of_range() {
        let o = Offset::Custom { minutes: -1440 };
        assert!(o.validate().is_err());
    }

    #[test]
    fn offset_max_valid() {
        let o = Offset::Custom { minutes: 1439 };
        assert!(o.validate().is_ok());
    }
}

mod parse_str {
    use super::*;

    #[test]
    fn offset_datetime() {
        let dt: Datetime = "1979-05-27T07:32:00Z".parse().expect("input should parse");
        assert_eq!(dt.date.as_ref().expect("field should be set").year, 1979);
        assert_eq!(dt.date.as_ref().expect("field should be set").month, 5);
        assert_eq!(dt.date.as_ref().expect("field should be set").day, 27);
        assert_eq!(dt.time.as_ref().expect("field should be set").hour, 7);
        assert_eq!(dt.time.as_ref().expect("field should be set").minute, 32);
        assert_eq!(dt.time.as_ref().expect("field should be set").second, 0);
        assert_eq!(dt.offset, Some(Offset::Z));
    }

    #[test]
    fn offset_datetime_with_offset() {
        let dt: Datetime = "1979-05-27T07:32:00+09:00"
            .parse()
            .expect("input should parse");
        assert_eq!(dt.offset, Some(Offset::Custom { minutes: 540 }));
    }

    #[test]
    fn offset_datetime_negative_offset() {
        let dt: Datetime = "1979-05-27T07:32:00-05:30"
            .parse()
            .expect("input should parse");
        assert_eq!(dt.offset, Some(Offset::Custom { minutes: -330 }));
    }

    #[test]
    fn local_datetime() {
        let dt: Datetime = "1979-05-27T07:32:00".parse().expect("input should parse");
        assert!(dt.date.is_some());
        assert!(dt.time.is_some());
        assert!(dt.offset.is_none());
    }

    #[test]
    fn local_datetime_with_space() {
        let dt: Datetime = "1979-05-27 07:32:00".parse().expect("input should parse");
        assert!(dt.date.is_some());
        assert!(dt.time.is_some());
    }

    #[test]
    fn local_date() {
        let dt: Datetime = "1979-05-27".parse().expect("input should parse");
        assert!(dt.date.is_some());
        assert!(dt.time.is_none());
    }

    #[test]
    fn local_time() {
        let dt: Datetime = "07:32:00".parse().expect("input should parse");
        assert!(dt.date.is_none());
        assert!(dt.time.is_some());
    }

    #[test]
    fn fractional_seconds() {
        let dt: Datetime = "07:32:00.123456789".parse().expect("input should parse");
        assert_eq!(
            dt.time.as_ref().expect("field should be set").nanosecond,
            123456789
        );
    }

    #[test]
    fn fractional_seconds_short() {
        let dt: Datetime = "07:32:00.1".parse().expect("input should parse");
        assert_eq!(
            dt.time.as_ref().expect("field should be set").nanosecond,
            100000000
        );
    }

    #[test]
    fn trailing_chars_error() {
        let result: Result<Datetime, _> = "07:32:00extra".parse();
        assert!(result.is_err());
    }

    #[test]
    fn invalid_format_error() {
        let result: Result<Datetime, _> = "not-a-date".parse();
        assert!(result.is_err());
    }

    #[test]
    fn lowercase_t_delimiter_is_valid() {
        let result: Result<Datetime, _> = "1979-05-27t07:32:00Z".parse();
        assert!(result.is_ok());
    }

    #[test]
    fn lowercase_z_offset_is_valid() {
        let result: Result<Datetime, _> = "1979-05-27T07:32:00z".parse();
        assert!(result.is_ok());
    }

    #[test]
    fn offset_minute_60_is_invalid() {
        let result: Result<Datetime, _> = "1979-05-27T07:32:00+00:60".parse();
        assert!(result.is_err());
    }

    #[test]
    fn offset_minute_99_is_invalid() {
        let result: Result<Datetime, _> = "1979-05-27T07:32:00+00:99".parse();
        assert!(result.is_err());
    }

    #[test]
    fn offset_minute_60_with_hours_is_invalid() {
        let result: Result<Datetime, _> = "1979-05-27T07:32:00+10:60".parse();
        assert!(result.is_err());
    }
}

mod display {
    use super::*;

    #[test]
    fn date_display() {
        let d = Date {
            year: 2024,
            month: 1,
            day: 5,
        };
        assert_eq!(d.to_string(), "2024-01-05");
    }

    #[test]
    fn time_display_no_fraction() {
        let t = Time {
            hour: 7,
            minute: 32,
            second: 0,
            nanosecond: 0,
        };
        assert_eq!(t.to_string(), "07:32:00");
    }

    #[test]
    fn time_display_with_fraction() {
        let t = Time {
            hour: 7,
            minute: 32,
            second: 0,
            nanosecond: 123000000,
        };
        assert_eq!(t.to_string(), "07:32:00.123");
    }

    #[test]
    fn offset_z_display() {
        assert_eq!(Offset::Z.to_string(), "Z");
    }

    #[test]
    fn offset_positive_display() {
        let o = Offset::Custom { minutes: 540 };
        assert_eq!(o.to_string(), "+09:00");
    }

    #[test]
    fn offset_negative_display() {
        let o = Offset::Custom { minutes: -330 };
        assert_eq!(o.to_string(), "-05:30");
    }
}
