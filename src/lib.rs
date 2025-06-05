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
    let input = eat_whitespace(input); // Skip leading whitespace for this value

    if input.starts_with("false") {
        return (Value::Boolean(false), &input["false".len()..]);
    }
    if input.starts_with("null") {
        return (Value::Null, &input["null".len()..]);
    }
    if input.starts_with("true") {
        return (Value::Boolean(true), &input["true".len()..]);
    }
    if input.starts_with('{') {
        let v = object(input);
        return (Value::Object(v.0), v.1); // object will return (HashMap, &str)
    }
    if input.starts_with('[') {
        let v = array(input);
        return (Value::Array(v.0), v.1); // object will return (HashMap, &str)
    }
    if input.starts_with('"') {
        return string(input).map(|s| Value::String(s)); // string returns (String, &str), convert to Value::String
    }
    // For numbers, you'd need a more complex check, e.g., regex or char-by-char
    // For now, let's assume it's a number if it starts with a digit or '-'
    if input.starts_with('-') || input.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        return number(input).map(|n| Value::Number(n)); // number returns (f64, &str), convert to Value::Number
    }

    panic!("Unexpected token: '{}'", input); // If nothing matches
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
    let mut obj: HashMap<String, Value> = HashMap::new();
    let mut cur_input = input;

    if !cur_input.starts_with('{') {
        panic!("object must start with '{{'.");
    }
    cur_input = &cur_input[1..];

    cur_input = eat_whitespace(cur_input);
    if let Some(rest) = cur_input.strip_prefix('}') {
        return (obj, rest);
    }

    // Parse key
    let (key, rest) = string(cur_input);
    cur_input = eat_whitespace(rest);
    if !cur_input.starts_with(':') {
        panic!("Expected ':' after object key.");
    }
    cur_input = eat_whitespace(&cur_input[1..]);

    // Parse value
    let (val, rest) = value(cur_input);
    obj.insert(key, val);
    cur_input = eat_whitespace(rest);

    if cur_input.starts_with(',') || cur_input.starts_with('}') {
        cur_input = &cur_input[1..];
    } else {
        panic!("Expected ',' or '}}' after object value.");
    }

    (obj, cur_input)
}

fn array(input: &str) -> (Vec<Value>, &str) {
    unimplemented!();
}

fn string(input: &str) -> (String, &str) {
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

    // `current_byte_pos` tracks the byte index *after* the character just processed.
    // It starts after the opening quote.
    let mut current_byte_pos = start_quote_idx + c.len_utf8();

    loop {
        let Some((char_idx, c)) = chars.next() else {
            // Reached end of input without finding closing quote
            panic!("Unterminated string: missing closing '\"'.");
        };
        current_byte_pos = char_idx + c.len_utf8(); // Update position to *after* the current char

        match c {
            '"' => {
                // Found the closing double quote
                return (parsed_string, &input[current_byte_pos..]);
            }
            '\\' => {
                // Handle escape sequence
                let Some((escaped_char_idx, escaped_char)) = chars.next() else {
                    panic!("Invalid escape sequence: '\\' at end of string.");
                };
                current_byte_pos = escaped_char_idx + escaped_char.len_utf8(); // Update position after escaped char

                match escaped_char {
                    '"' => parsed_string.push('"'),
                    '\\' => parsed_string.push('\\'),
                    '/' => parsed_string.push('/'),
                    'b' => parsed_string.push('\x08'), // Backspace
                    'f' => parsed_string.push('\x0C'), // Form feed
                    'n' => parsed_string.push('\n'),
                    'r' => parsed_string.push('\r'),
                    't' => parsed_string.push('\t'),
                    'u' => {
                        // Unicode escape \uXXXX
                        let mut hex_val: u32 = 0;
                        for _ in 0..4 {
                            let Some((hex_char_idx, hex_c)) = chars.next() else {
                                panic!(
                                    "Invalid unicode escape sequence: expected 4 hex digits after '\\u'."
                                );
                            };
                            current_byte_pos = hex_char_idx + hex_c.len_utf8(); // Update position

                            let digit = hex_c
                                .to_digit(16)
                                .expect("Invalid hex digit in unicode escape.");
                            hex_val = (hex_val << 4) | digit;
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

// Helper trait to allow .map() on tuples, making value() cleaner
trait MapTuple<T, U, V> {
    fn map<F>(self, f: F) -> (V, U)
    where
        F: FnOnce(T) -> V;
}

impl<T, U> MapTuple<T, U, Value> for (T, U) {
    fn map<F>(self, f: F) -> (Value, U)
    where
        F: FnOnce(T) -> Value,
    {
        (f(self.0), self.1)
    }
}

fn number(input: &str) -> (f64, &str) {
    unimplemented!();
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
        let json = r#""hello \"world\"\\/\b\f\n\r\t\u0041""#;
        let parsed = parse(json);
        match parsed {
            Value::String(s) => {
                assert_eq!(s, "hello \"world\"/\\/\x08\x0C\n\r\tA");
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
    #[should_panic(expected = "Expected ',' or '}}' after object value.")]
    fn parse_object_missing_comma_or_brace() {
        let json = r#"{"key": "value" "another_key": "another_value"}"#;
        parse(json);
    }
}
