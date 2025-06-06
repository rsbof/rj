// Defined in RFC8259 also known as STD90.

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
}

pub fn parse(input: &str) -> Value {
    let (v, rest) = value(input);
    // After parsing the top-level value, there should ideally be only whitespace left.
    let rest = eat_whitespace(rest);
    if !rest.is_empty() {
        panic!("Unexpected characters after JSON value: '{}'", rest);
    }
    v
}

fn value(input: &str) -> (Value, &str) {
    let input = eat_whitespace(input);

    if let Some(rest) = input.strip_prefix("false") {
        return (Value::Boolean(false), rest);
    }
    if let Some(rest) = input.strip_prefix("null") {
        return (Value::Null, rest);
    }
    if let Some(rest) = input.strip_prefix("true") {
        return (Value::Boolean(true), rest);
    }
    if input.starts_with('{') {
        let v = object(input);
        return (Value::Object(v.0), v.1);
    }
    if input.starts_with('[') {
        let v = array(input);
        return (Value::Array(v.0), v.1);
    }
    if input.starts_with('"') {
        let v = string(input);
        return (Value::String(v.0), v.1);
    }
    if input.starts_with('-') || input.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        let v = number(input);
        return (Value::Number(v.0), v.1);
    }

    panic!("Unexpected token: '{}'", input);
}

fn eat_whitespace(input: &str) -> &str {
    let mut pos = 0;
    for c in input.chars() {
        if !c.is_whitespace() {
            break;
        }
        pos += c.len_utf8(); // Advance by byte length of char
    }
    &input[pos..]
}

fn object(input: &str) -> (HashMap<String, Value>, &str) {
    let mut cur_input = eat_whitespace(input)
        .strip_prefix('{')
        .expect("object must start with '{'");

    if let Some(rest) = eat_whitespace(cur_input).strip_prefix('}') {
        return (HashMap::new(), rest);
    }

    let mut obj: HashMap<String, Value> = HashMap::new();
    loop {
        // Parse key
        let (key, rest) = string(eat_whitespace(cur_input));
        cur_input = eat_whitespace(rest)
            .strip_prefix(':')
            .expect("Expected ':' after object key.");

        // Parse value
        let (val, rest) = value(cur_input);
        obj.insert(key, val);

        if let Some(rest) = eat_whitespace(rest).strip_prefix(',') {
            cur_input = rest;
        } else if let Some(rest) = eat_whitespace(rest).strip_prefix('}') {
            cur_input = rest;
            break;
        } else {
            panic!("Expected ',' or '}}' after object value.");
        }
    }

    (obj, cur_input)
}

fn array(input: &str) -> (Vec<Value>, &str) {
    let mut cur_input = eat_whitespace(input)
        .strip_prefix('[')
        .expect("array must start with '['");

    if let Some(rest) = eat_whitespace(cur_input).strip_prefix(']') {
        return (Vec::new(), rest);
    }

    let mut values: Vec<Value> = Vec::new();
    let (v, rest) = value(cur_input);
    values.push(v);
    cur_input = rest;

    while let Some(rest) = eat_whitespace(cur_input).strip_prefix(',') {
        let (v, rest) = value(rest);
        values.push(v);
        cur_input = rest;
    }

    cur_input = eat_whitespace(cur_input)
        .strip_prefix(']')
        .expect("array must end with ']'");

    (values, cur_input)
}

fn string(input: &str) -> (String, &str) {
    let cur_input = eat_whitespace(input)
        .strip_prefix('"')
        .expect("object must start with '\"'");

    if let Some(rest) = eat_whitespace(cur_input).strip_prefix('"') {
        return (String::new(), rest);
    }

    let mut chars = input.char_indices(); // Iterator that yields (byte_index, char)
    let mut parsed_string = String::new();

    // 1. Expect the opening double quote
    let Some((start_quote_idx, c)) = chars.next() else {
        panic!("String must start with '\"'. Input was empty.");
    };
    if c != '"' {
        panic!(
            "String must start with '\"'. Found '{}' at index {}.",
            c, start_quote_idx
        );
    }

    loop {
        let Some((idx, c)) = chars.next() else {
            panic!("Unterminated string: missing closing '\"'.");
        };
        // `current_byte_pos` tracks the byte index *after* the character just processed.
        // It starts after the opening quote.
        let current_byte_pos = idx + c.len_utf8(); // Update position to *after* the current char

        match c {
            '"' => {
                return (parsed_string, &input[current_byte_pos..]);
            }
            '\\' => {
                // Handle escape sequence
                let Some((_, escaped_char)) = chars.next() else {
                    panic!("Invalid escape sequence: '\\' at end of string.");
                };

                match escaped_char {
                    '"' => parsed_string.push('"'),    // quotation mark
                    '\\' => parsed_string.push('\\'),  // reverse solidus
                    '/' => parsed_string.push('/'),    // solidus
                    'b' => parsed_string.push('\x08'), // backspace
                    'f' => parsed_string.push('\x0C'), // form feed
                    'n' => parsed_string.push('\n'),   // line feed
                    'r' => parsed_string.push('\r'),   // carriage return
                    't' => parsed_string.push('\t'),   // tab
                    'u' => {
                        // uXXXX
                        let mut hex_val: u32 = 0;
                        for _ in 0..4 {
                            match chars.next() {
                                Some((_, '"')) => {
                                    panic!(
                                        "Invalid unicode escape sequence: expected 4 hex digits after '\\u'."
                                    );
                                }
                                Some((_, c)) => {
                                    let digit = c
                                        .to_digit(16)
                                        .expect("Invalid hex digit in unicode escape.");
                                    hex_val = (hex_val << 4) | digit;
                                }
                                None => {
                                    panic!(
                                        "Invalid unicode escape sequence: expected 4 hex digits after '\\u'."
                                    );
                                }
                            }
                        }

                        let unicode_char =
                            char::from_u32(hex_val).expect("Invalid unicode scalar value.");
                        parsed_string.push(unicode_char);
                    }
                    _ => panic!("Invalid escape sequence: '\\{}'", escaped_char),
                }
            }
            // JSON strings cannot contain unescaped control characters like newlines or carriage returns
            _ if c == '\n' || c == '\r' || c == '\t' => {
                // \t is allowed escaped, but not unescaped
                panic!("Unescaped control character in string: '{}'", c);
            }
            _ => {
                // Regular character
                parsed_string.push(c);
            }
        }
    }
}

fn number(input: &str) -> (f64, &str) {
    // ignore whitespace first
    let mut cur_input = eat_whitespace(input);

    let mut minus = false;
    if let Some(rest) = cur_input.strip_prefix('-') {
        minus = true;
        cur_input = rest;
    }

    let mut buf = String::new();
    for c in cur_input.chars() {
        match c {
            '0'..='9' => buf.push(c),
            '.' => buf.push(c),
            'e' | 'E' => buf.push(c),
            _ => break, // the char is not part of number.
        }
    }

    cur_input = cur_input.strip_prefix(&buf).unwrap();
    if minus {
        (buf.parse::<f64>().unwrap() * -1.0, cur_input)
    } else {
        (buf.parse().unwrap(), cur_input)
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_object() {
        let json = "{}";
        let parsed = parse(json);
        match parsed {
            Value::Object(obj) => {
                assert!(obj.is_empty());
            }
            _ => panic!("Expected an object, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_object_with_whitespace() {
        let json = "{   }";
        let parsed = parse(json);
        match parsed {
            Value::Object(obj) => {
                assert!(obj.is_empty());
            }
            _ => panic!("Expected an object, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_simple_string() {
        let json = r#""hello""#;
        let parsed = parse(json);
        match parsed {
            Value::String(s) => {
                assert_eq!(s, "hello");
            }
            _ => panic!("Expected a string, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_string_with_escapes() {
        let json = r#""hello \"world\"\\\/\b\f\n\r\t\u0041""#;
        let parsed = parse(json);
        match parsed {
            Value::String(s) => {
                assert_eq!(s, "hello \"world\"\\/\x08\x0c\x0a\x0d\tA");
            }
            _ => panic!("Expected a string, got {:?}", parsed),
        }
    }

    #[test]
    #[should_panic(expected = "Unterminated string: missing closing '\"'.")]
    fn parse_unterminated_string() {
        let json = r#""hello"#;
        parse(json);
    }

    #[test]
    #[should_panic(expected = "Invalid escape sequence: '\\x'")]
    fn parse_string_with_invalid_escape() {
        let json = r#""hello\x""#;
        parse(json);
    }

    #[test]
    #[should_panic(
        expected = "Invalid unicode escape sequence: expected 4 hex digits after '\\u'."
    )]
    fn parse_string_with_incomplete_unicode_escape() {
        let json = r#""\u123""#;
        parse(json);
    }

    #[test]
    #[should_panic(expected = "Invalid hex digit in unicode escape.")]
    fn parse_string_with_invalid_unicode_hex() {
        let json = r#""\u123G""#;
        parse(json);
    }

    #[test]
    fn parse_string_with_valid_unicode_hex() {
        let json = r#""\u3042""#;
        let parsed = parse(json);
        match parsed {
            Value::String(s) => {
                assert_eq!(s.len(), 3);
                assert_eq!(s, "あ".to_string());
            }
            _ => panic!("Expected an string, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_object_with_one_string_member() {
        let json = r#"{"key": "value"}"#;
        let parsed = parse(json);
        match parsed {
            Value::Object(obj) => {
                assert_eq!(obj.len(), 1);
                assert_eq!(obj.get("key"), Some(&Value::String("value".to_string())));
            }
            _ => panic!("Expected an object, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_object_with_multiple_string_members() {
        let json = r#"{ "key1" : "value1" , "key2" : "value2" }"#;
        let parsed = parse(json);
        match parsed {
            Value::Object(obj) => {
                assert_eq!(obj.len(), 2);
                assert_eq!(obj.get("key1"), Some(&Value::String("value1".to_string())));
                assert_eq!(obj.get("key2"), Some(&Value::String("value2".to_string())));
            }
            _ => panic!("Expected an object, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_object_with_boolean_members() {
        let json = r#"{"t": true, "f": false, "n": null}"#;
        let parsed = parse(json);
        match parsed {
            Value::Object(obj) => {
                assert_eq!(obj.len(), 3);
                assert_eq!(obj.get("t"), Some(&Value::Boolean(true)));
                assert_eq!(obj.get("f"), Some(&Value::Boolean(false)));
                assert_eq!(obj.get("n"), Some(&Value::Null));
            }
            _ => panic!("Expected an object, got {:?}", parsed),
        }
    }

    #[test]
    #[should_panic(expected = "Unexpected characters after JSON value: 'extra'")]
    fn parse_extra_characters_after_value() {
        let json = r#"{}extra"#;
        parse(json);
    }

    #[test]
    #[should_panic(expected = "Expected ':' after object key.")]
    fn parse_object_missing_colon() {
        let json = r#"{"key" "value"}"#;
        parse(json);
    }

    #[test]
    #[should_panic(expected = "Expected ',' or '}' after object value.")]
    fn parse_object_missing_comma_or_brace() {
        let json = r#"{"key": "value" "another_key": "another_value"}"#;
        parse(json);
    }

    #[test]
    fn parse_number() {
        let json = r#"10"#;
        let parsed = parse(json);
        match parsed {
            Value::Number(n) => {
                assert_eq!(n, 10.0)
            }
            _ => panic!("Expected a number, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_number_with_minus_sign() {
        let json = r#"-10"#;
        let parsed = parse(json);
        match parsed {
            Value::Number(n) => {
                assert_eq!(n, -10.0)
            }
            _ => panic!("Expected a number, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_number_with_fraction() {
        let json = r#"10.01234"#;
        let parsed = parse(json);
        match parsed {
            Value::Number(n) => {
                assert_eq!(n, 10.01234)
            }
            _ => panic!("Expected a number, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_number_with_exponent() {
        let json = r#"10e3"#;
        let parsed = parse(json);
        match parsed {
            Value::Number(n) => {
                assert_eq!(n, 10000.0)
            }
            _ => panic!("Expected a number, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_array_with_empty() {
        let json = r#"[]"#;
        let parsed = parse(json);
        match parsed {
            Value::Array(arr) => assert_eq!(arr, vec![]),
            _ => panic!("Expected an array, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_array_with_single_object() {
        let json = r#"[{"key1": true}]"#;
        let parsed = parse(json);
        match parsed {
            Value::Array(arr) => assert_eq!(
                arr,
                vec![Value::Object(HashMap::from([(
                    "key1".to_string(),
                    Value::Boolean(true)
                )]))]
            ),
            _ => panic!("Expected an array, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_array_with_multiple_objects() {
        let json = r#"[{"key1": true}, {"key1": true}]"#;
        let parsed = parse(json);
        match parsed {
            Value::Array(arr) => assert_eq!(
                arr,
                vec![
                    Value::Object(HashMap::from([("key1".to_string(), Value::Boolean(true))])),
                    Value::Object(HashMap::from([("key1".to_string(), Value::Boolean(true))])),
                ]
            ),
            _ => panic!("Expected an array, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_array_with_single_array() {
        let json = r#"[[]]"#;
        let parsed = parse(json);
        match parsed {
            Value::Array(arr) => assert_eq!(arr, vec![Value::Array(vec![])]),
            _ => panic!("Expected an array, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_array_with_multiple_arrays() {
        let json = r#"[[],[],[]]"#;
        let parsed = parse(json);
        match parsed {
            Value::Array(arr) => assert_eq!(
                arr,
                vec![
                    Value::Array(vec![]),
                    Value::Array(vec![]),
                    Value::Array(vec![]),
                ]
            ),
            _ => panic!("Expected an array, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_array_with_nested_arrays() {
        let json = r#"[[[]]]"#;
        let parsed = parse(json);
        match parsed {
            Value::Array(arr) => assert_eq!(arr, vec![Value::Array(vec![Value::Array(vec![])]),]),
            _ => panic!("Expected an array, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_example1_in_rfc8259() {
        let json = r#"
{
    "Image": {
        "Width":  800,
        "Height": 600,
        "Title":  "View from 15th Floor",
        "Thumbnail": {
            "Url":    "http://www.example.com/image/481989943",
            "Height": 125,
            "Width":  100
        },
        "Animated" : false,
        "IDs": [116, 943, 234, 38793]
    }
}
"#;
        let parsed = parse(json);
        match &parsed {
            Value::Object(obj) => match &obj["Image"] {
                Value::Object(obj) => {
                    assert_eq!(obj["Width"], Value::Number(800.0));
                    assert_eq!(obj["Height"], Value::Number(600.0));
                    assert_eq!(
                        obj["Title"],
                        Value::String("View from 15th Floor".to_string())
                    );
                    match &obj["Thumbnail"] {
                        Value::Object(obj) => {
                            assert_eq!(
                                obj["Url"],
                                Value::String("http://www.example.com/image/481989943".to_string())
                            );
                            assert_eq!(obj["Height"], Value::Number(125.0));
                            assert_eq!(obj["Width"], Value::Number(100.0));
                        }
                        _ => panic!("Expected an object, got {:?}", parsed),
                    }
                    assert_eq!(obj["Animated"], Value::Boolean(false));
                    assert_eq!(obj["Width"], Value::Number(800.0));
                }
                _ => panic!("Expected an object, got {:?}", parsed),
            },
            _ => panic!("Expected an object, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_example2_in_rfc8259() {
        let json = r#"
[
    {
        "precision": "zip",
        "Latitude":  37.7668,
        "Longitude": -122.3959,
        "Address":   "",
        "City":      "SAN FRANCISCO",
        "State":     "CA",
        "Zip":       "94107",
        "Country":   "US"
    },
    {
        "precision": "zip",
        "Latitude":  37.371991,
        "Longitude": -122.026020,
        "Address":   "",
        "City":      "SUNNYVALE",
        "State":     "CA",
        "Zip":       "94085",
        "Country":   "US"
    }
]
"#;
        let parsed = parse(json);
        match &parsed {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                match &arr[0] {
                    Value::Object(obj) => {
                        assert_eq!(obj["precision"], Value::String("zip".to_string()));
                        assert_eq!(obj["Latitude"], Value::Number(37.7668));
                        assert_eq!(obj["Longitude"], Value::Number(-122.3959));
                        assert_eq!(obj["Address"], Value::String("".to_string()));
                        assert_eq!(obj["City"], Value::String("SAN FRANCISCO".to_string()));
                        assert_eq!(obj["State"], Value::String("CA".to_string()));
                        assert_eq!(obj["Zip"], Value::String("94107".to_string()));
                        assert_eq!(obj["Country"], Value::String("US".to_string()));
                    }
                    _ => panic!("Expected an object, got {:?}", parsed),
                };
                match &arr[1] {
                    Value::Object(obj) => {
                        assert_eq!(obj["precision"], Value::String("zip".to_string()));
                        assert_eq!(obj["Latitude"], Value::Number(37.371991));
                        assert_eq!(obj["Longitude"], Value::Number(-122.026020));
                        assert_eq!(obj["Address"], Value::String("".to_string()));
                        assert_eq!(obj["City"], Value::String("SUNNYVALE".to_string()));
                        assert_eq!(obj["State"], Value::String("CA".to_string()));
                        assert_eq!(obj["Zip"], Value::String("94085".to_string()));
                        assert_eq!(obj["Country"], Value::String("US".to_string()));
                    }
                    _ => panic!("Expected an object, got {:?}", parsed),
                };
            }
            _ => panic!("Expected an array, got {:?}", parsed),
        }
    }
}
