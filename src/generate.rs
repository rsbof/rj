use crate::Value;

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(x) => write!(f, "\"{x}\""),
            Value::Number(x) => write!(f, "{x}"),
            Value::Boolean(x) => write!(f, "{x}"),
            Value::Null => write!(f, "null"),
            Value::Object(obj) => {
                let mut buf = String::new();
                buf.push('{');
                let mut members: Vec<String> = Vec::new();
                for (k, v) in obj.iter() {
                    members.push(format!("{}:{}", &Value::String(k.to_string()), v));
                }
                buf.push_str(&members.join(","));
                buf.push('}');
                write!(f, "{buf}")
            }
            Value::Array(arr) => {
                let mut buf = String::new();
                buf.push('[');
                let mut elements: Vec<String> = Vec::new();
                for v in arr {
                    elements.push(v.to_string());
                }
                buf.push_str(&elements.join(","));
                buf.push(']');
                write!(f, "{buf}")
            }
        }
    }
}

#[cfg(test)]
mod generate_tests {
    use super::*;

    #[test]
    fn string() {
        let json = r#""string""#;
        let s = Value::from(json).to_string();
        assert_eq!(s, json);
    }

    #[test]
    fn number() {
        let json = "10.1234";
        let s = Value::from(json).to_string();
        assert_eq!(s, json);
    }

    #[test]
    fn boolean() {
        let json = r#"false"#;
        let s = Value::from(json).to_string();
        assert_eq!(s, json);

        let json = r#"true"#;
        let s = Value::from(json).to_string();
        assert_eq!(s, json);
    }

    #[test]
    fn null() {
        let json = r#"null"#;
        let s = Value::from(json).to_string();
        assert_eq!(s, json);
    }

    #[test]
    fn array() {
        let json = r#"["string","string2"]"#;
        let s = Value::from(json).to_string();
        assert_eq!(s, json);
    }

    #[test]
    fn object() {
        let json = r#"{"key":"value"}"#;
        let s = Value::from(json).to_string();
        assert_eq!(s, json);
    }

    #[test]
    #[ignore = "order of keys is not guaranteed"]
    fn object_with_members() {
        let json = r#"{"key":"value","key2":"value2"}"#;
        let s = Value::from(json).to_string();
        assert_eq!(s, json);
    }
}
