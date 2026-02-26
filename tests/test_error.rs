use shiguredo_toml::Error;

mod parse_error {
    use super::*;

    #[test]
    fn position() {
        let err = Error::Parse {
            message: "test".into(),
            position: 10,
        };
        assert_eq!(err.position(), Some(10));
    }

    #[test]
    fn serialize_error_no_position() {
        let err = Error::Serialize {
            message: "test".into(),
        };
        assert_eq!(err.position(), None);
    }

    #[test]
    fn line_and_column() {
        let text = "abc\ndef\nghi";
        let err = Error::Parse {
            message: "test".into(),
            position: 5, // 'e' in "def"
        };
        let (line, col) = err.get_line_and_column(text).unwrap();
        assert_eq!(line.get(), 2);
        assert_eq!(col.get(), 2);
    }

    #[test]
    fn line_and_column_first_line() {
        let text = "abcdef";
        let err = Error::Parse {
            message: "test".into(),
            position: 0,
        };
        let (line, col) = err.get_line_and_column(text).unwrap();
        assert_eq!(line.get(), 1);
        assert_eq!(col.get(), 1);
    }

    #[test]
    fn get_line() {
        let text = "abc\ndefgh\nijk";
        let err = Error::Parse {
            message: "test".into(),
            position: 5,
        };
        assert_eq!(err.get_line(text), Some("defgh"));
    }

    #[test]
    fn display_parse_error() {
        let err = Error::Parse {
            message: "test error".into(),
            position: 5,
        };
        let s = format!("{err}");
        assert!(s.contains("parse error"));
        assert!(s.contains("byte 5"));
        assert!(s.contains("test error"));
    }

    #[test]
    fn display_serialize_error() {
        let err = Error::Serialize {
            message: "test error".into(),
        };
        let s = format!("{err}");
        assert!(s.contains("serialize error"));
        assert!(s.contains("test error"));
    }

    #[test]
    fn implements_std_error() {
        let err = Error::Parse {
            message: "test".into(),
            position: 0,
        };
        let _: &dyn std::error::Error = &err;
    }
}
