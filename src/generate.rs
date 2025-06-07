use crate::Value;

pub(crate) fn generate(value: &Value) -> String {
    match value {
        Value::String(x) => format!("\"{x}\""),
        Value::Number(x) => x.to_string(),
        Value::Boolean(x) => x.to_string(),
        Value::Null => "null".to_string(),
        Value::Object(obj) => {
            let mut buf = String::new();
            buf.push('{');
            let mut members: Vec<String> = Vec::new();
            for (k, v) in obj.iter() {
                members.push(format!(
                    "{}:{}",
                    generate(&Value::String(k.to_string())),
                    generate(v)
                ));
            }
            buf.push_str(&members.join(","));
            buf.push('}');
            buf
        }
        Value::Array(arr) => {
            let mut buf = String::new();
            buf.push('[');
            for v in arr {
                buf.push_str(generate(v).as_str());
                buf.push(',');
            }
            buf.pop();
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
        let generated = generate(&json.into());
        assert_eq!(generated, json);
    }

    #[test]
    fn number() {
        let v = Value::Number(10.1234);
        let json = "10.1234";
        let generated = generate(&v);
        assert_eq!(generated, json);
    }

    #[test]
    fn boolean() {
        let json = r#"false"#;
        let generated = generate(&json.into());
        assert_eq!(generated, json);

        let json = r#"true"#;
        let generated = generate(&json.into());
        assert_eq!(generated, json);
    }

    #[test]
    fn null() {
        let json = r#"null"#;
        let generated = generate(&json.into());
        assert_eq!(generated, json);
    }

    #[test]
    fn array() {
        let json = r#"["string","string2"]"#;
        let generated = generate(&json.into());
        assert_eq!(generated, json);
    }

    #[test]
    fn object() {
        let json = r#"{"key":"value"}"#;
        let generated = generate(&json.into());
        assert_eq!(generated, json);
    }

    #[test]
    #[ignore = "order of keys is not guaranteed"]
    fn object_with_members() {
        let json = r#"{"key":"value","key2":"value2"}"#;
        let generated = generate(&json.into());
        assert_eq!(generated, json);
    }
}
