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

pub(crate) fn format(value: &Value, indent: usize) -> String {
    match value {
        Value::String(x) => format!("\"{x}\""),
        Value::Number(x) => x.to_string(),
        Value::Boolean(x) => x.to_string(),
        Value::Null => "null".to_string(),
        Value::Object(obj) => {
            if obj.is_empty() {
                return "{}".to_string();
            }
            let mut buf = String::new();
            buf.push_str("{\n");
            buf.push_str(&" ".repeat(indent));
            for (i, (k, v)) in obj.iter().enumerate() {
                buf.push_str(&format!("\"{k}\""));
                buf.push_str(": ");
                buf.push_str(&format(v, indent + 2));
                if i < obj.len() - 1 {
                    buf.push_str(",\n");
                    buf.push_str(&" ".repeat(indent));
                }
            }
            buf.push('\n');
            buf.push_str(&" ".repeat(indent - 2));
            buf.push('}');
            buf
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let mut buf = String::new();
            buf.push('[');
            buf.push('\n');
            buf.push_str(&" ".repeat(indent));
            for (i, ele) in arr.iter().enumerate() {
                buf.push_str(&format(ele, indent + 2));
                if i < arr.len() - 1 {
                    buf.push_str(",\n");
                    buf.push_str(&" ".repeat(indent));
                }
            }
            buf.push('\n');
            buf.push_str(&" ".repeat(indent - 2));
            buf.push(']');
            buf
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
    fn object_with_members() {
        let json = r#"{"key":"value","key2":"value2"}"#;
        let s = Value::from(json).to_string();
        assert_eq!(s, json);
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;

    #[test]
    fn test_object() {
        let json = r#"{"key":"value"}"#;
        let formatted = format(&json.into(), 2);
        assert_eq!(formatted, "{\n  \"key\": \"value\"\n}");
    }

    #[test]
    fn test_nested_object() {
        let json = r#"{"key":{"key2":"value2"}}"#;
        let formatted = format(&json.into(), 2);
        assert_eq!(
            formatted,
            "{\n  \"key\": {\n    \"key2\": \"value2\"\n  }\n}"
        );
    }

    #[test]
    fn test_empty_object() {
        let json = r#"{}"#;
        let formatted = format(&json.into(), 2);
        assert_eq!(formatted, "{}");
    }

    #[test]
    fn test_nested_empty_object() {
        let json = r#"{"key": {}}"#;
        let formatted = format(&json.into(), 2);
        assert_eq!(formatted, "{\n  \"key\": {}\n}");
    }

    #[test]
    fn test_array() {
        let json = r#"[1,2,3]"#;
        let formatted = format(&json.into(), 2);
        assert_eq!(formatted, "[\n  1,\n  2,\n  3\n]");
    }

    #[test]
    fn test_nested_array() {
        let json = r#"[1,[2,[3]]]"#;
        let formatted = format(&json.into(), 2);
        assert_eq!(
            formatted,
            "[\n  1,\n  [\n    2,\n    [\n      3\n    ]\n  ]\n]"
        );
    }

    #[test]
    fn test_empty_array() {
        let json = r#"[]"#;
        let formatted = format(&json.into(), 2);
        assert_eq!(formatted, "[]");
    }

    #[test]
    fn test_nested_empty_array() {
        let json = r#"[[],[[]]]"#;
        let formatted = format(&json.into(), 2);
        assert_eq!(formatted, "[\n  [],\n  [\n    []\n  ]\n]");
    }
}
