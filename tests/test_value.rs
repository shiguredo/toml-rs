use shiguredo_toml::{Table, Value};

mod type_checks {
    use super::*;

    #[test]
    fn is_methods() {
        assert!(Value::String("".into()).is_str());
        assert!(Value::Integer(0).is_integer());
        assert!(Value::Float(0.0).is_float());
        assert!(Value::Boolean(true).is_bool());
        assert!(Value::Array(vec![]).is_array());
        assert!(Value::Table(Table::new()).is_table());
    }

    #[test]
    fn as_methods_some() {
        assert_eq!(Value::String("hello".into()).as_str(), Some("hello"));
        assert_eq!(Value::Integer(42).as_integer(), Some(42));
        assert_eq!(Value::Float(1.5).as_float(), Some(1.5));
        assert_eq!(Value::Boolean(true).as_bool(), Some(true));
    }

    #[test]
    fn as_methods_none() {
        assert!(Value::Integer(42).as_str().is_none());
        assert!(Value::String("".into()).as_integer().is_none());
        assert!(Value::String("".into()).as_float().is_none());
        assert!(Value::String("".into()).as_bool().is_none());
        assert!(Value::String("".into()).as_array().is_none());
        assert!(Value::String("".into()).as_table().is_none());
        assert!(Value::String("".into()).as_datetime().is_none());
    }
}

mod access {
    use super::*;

    #[test]
    fn get_existing_key() {
        let mut t = Table::new();
        t.insert("key".into(), Value::Integer(1));
        let v = Value::Table(t);
        assert_eq!(v.get("key").unwrap().as_integer().unwrap(), 1);
    }

    #[test]
    fn get_missing_key() {
        let v = Value::Table(Table::new());
        assert!(v.get("missing").is_none());
    }

    #[test]
    fn get_on_non_table() {
        let v = Value::Integer(42);
        assert!(v.get("key").is_none());
    }

    #[test]
    fn index_str() {
        let mut t = Table::new();
        t.insert("key".into(), Value::Integer(1));
        let v = Value::Table(t);
        assert_eq!(v["key"].as_integer().unwrap(), 1);
    }

    #[test]
    fn index_usize() {
        let v = Value::Array(vec![Value::Integer(10), Value::Integer(20)]);
        assert_eq!(v[0].as_integer().unwrap(), 10);
        assert_eq!(v[1].as_integer().unwrap(), 20);
    }

    #[test]
    #[should_panic(expected = "キー")]
    fn index_str_missing_panics() {
        let v = Value::Table(Table::new());
        let _ = &v["missing"];
    }

    #[test]
    #[should_panic]
    fn index_usize_on_non_array_panics() {
        let v = Value::Integer(42);
        let _ = &v[0];
    }
}

mod conversions {
    use super::*;

    #[test]
    fn from_string() {
        let v: Value = "hello".into();
        assert_eq!(v.as_str().unwrap(), "hello");
    }

    #[test]
    fn from_owned_string() {
        let v: Value = String::from("hello").into();
        assert_eq!(v.as_str().unwrap(), "hello");
    }

    #[test]
    fn from_i64() {
        let v: Value = 42i64.into();
        assert_eq!(v.as_integer().unwrap(), 42);
    }

    #[test]
    fn from_i32() {
        let v: Value = 42i32.into();
        assert_eq!(v.as_integer().unwrap(), 42);
    }

    #[test]
    fn from_f64() {
        let v: Value = 2.72f64.into();
        assert_eq!(v.as_float().unwrap(), 2.72);
    }

    #[test]
    fn from_bool() {
        let v: Value = true.into();
        assert!(v.as_bool().unwrap());
    }
}

mod from_str {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn value_from_str() {
        let v = Value::from_str("a = 1").unwrap();
        assert!(v.is_table());
    }

    #[test]
    fn value_from_str_error() {
        let result = Value::from_str("invalid = ");
        assert!(result.is_err());
    }
}
