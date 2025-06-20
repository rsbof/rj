use indexmap::IndexMap;

use crate::Value;

type ValueAndRest<'a> = (Value, &'a str);
pub type Result<T> = std::result::Result<T, self::Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedToken(String),
    MissingExpectedChar(char, String),
    UnterminatedString,
    InvalidEscapeSequence(String),
    InvalidUnicodeEscape,
    InvalidNumberFormat(String),
    TrailingCharacters(String),
}

pub(crate) fn parse(input: &str) -> Result<Value> {
    let (v, rest) = value(input)?;
    let rest = eat_whitespace(rest);
    if !rest.is_empty() {
        return Err(Error::TrailingCharacters(format!(
            "Unexpected characters after JSON value: '{}'",
            rest
        )));
    }
    Ok(v)
}

fn value(input: &str) -> Result<ValueAndRest> {
    let input = eat_whitespace(input);

    if let Some(rest) = input.strip_prefix("false") {
        return Ok((Value::Boolean(false), rest));
    }
    if let Some(rest) = input.strip_prefix("null") {
        return Ok((Value::Null, rest));
    }
    if let Some(rest) = input.strip_prefix("true") {
        return Ok((Value::Boolean(true), rest));
    }
    if input.starts_with('{') {
        let (v, rest) = object(input)?;
        return Ok((v, rest));
    }
    if input.starts_with('[') {
        let (v, rest) = array(input)?;
        return Ok((v, rest));
    }
    if input.starts_with('"') {
        let (v, rest) = string(input)?;
        return Ok((v, rest));
    }
    if input.starts_with('-') || input.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        let (v, rest) = number(input)?;
        return Ok((v, rest));
    }

    Err(Error::UnexpectedToken(format!(
        "Unexpected token: '{}'",
        input
    )))
}

/// whitespace = \x20 \x09 \x0a \x0d
/// - \x20 space
/// - \x09 horizontal tab
/// - \x0a line feed or new line
/// - \x0d carriage return
fn is_whitespace(c: char) -> bool {
    c == '\x20' || c == '\x09' || c == '\x0a' || c == '\x0d'
}

fn eat_whitespace(input: &str) -> &str {
    let mut pos = 0;
    for c in input.chars() {
        if !is_whitespace(c) {
            break;
        }
        pos += c.len_utf8(); // Advance by byte length of char
    }
    &input[pos..]
}

fn object(input: &str) -> Result<ValueAndRest> {
    let mut cur_input = eat_whitespace(input)
        .strip_prefix('{')
        .ok_or_else(|| Error::MissingExpectedChar('{', input.to_string()))?;

    if let Some(rest) = eat_whitespace(cur_input).strip_prefix('}') {
        return Ok((Value::Object(IndexMap::new()), rest));
    }

    let mut obj: IndexMap<String, Value> = IndexMap::new();
    loop {
        // Parse key
        let (v, rest) = string(eat_whitespace(cur_input))?;
        let key = match v {
            Value::String(s) => s,
            _ => unreachable!("string() should always return Value::String"),
        };

        cur_input = eat_whitespace(rest)
            .strip_prefix(':')
            .ok_or_else(|| Error::MissingExpectedChar(':', rest.to_string()))?;

        // Parse value
        let (v, rest) = value(cur_input)?;
        obj.insert(key, v);

        if let Some(rest) = eat_whitespace(rest).strip_prefix(',') {
            cur_input = rest;
        } else if let Some(rest) = eat_whitespace(rest).strip_prefix('}') {
            cur_input = rest;
            break;
        } else {
            return Err(Error::UnexpectedToken(format!(
                "Expected ',' or '}}' after object value. Found: '{}'",
                rest
            )));
        }
    }

    Ok((Value::Object(obj), cur_input))
}

fn array(input: &str) -> Result<ValueAndRest> {
    let mut cur_input = eat_whitespace(input)
        .strip_prefix('[')
        .ok_or_else(|| Error::MissingExpectedChar('[', input.to_string()))?;

    if let Some(rest) = eat_whitespace(cur_input).strip_prefix(']') {
        return Ok((Value::Array(Vec::new()), rest));
    }

    let mut values: Vec<Value> = Vec::new();
    let (v, rest) = value(cur_input)?;
    values.push(v);
    cur_input = rest;

    while let Some(rest) = eat_whitespace(cur_input).strip_prefix(',') {
        let (v, rest) = value(rest)?;
        values.push(v);
        cur_input = rest;
    }

    cur_input = eat_whitespace(cur_input)
        .strip_prefix(']')
        .ok_or_else(|| Error::MissingExpectedChar(']', cur_input.to_string()))?;

    Ok((Value::Array(values), cur_input))
}

fn string(input: &str) -> Result<ValueAndRest> {
    let cur_input = eat_whitespace(input)
        .strip_prefix('"')
        .ok_or_else(|| Error::MissingExpectedChar('"', input.to_string()))?;

    if let Some(rest) = eat_whitespace(cur_input).strip_prefix('"') {
        return Ok((Value::String(String::new()), rest));
    }

    let mut chars = cur_input.char_indices();
    let mut parsed_string = String::new();

    loop {
        let Some((idx, c)) = chars.next() else {
            return Err(Error::UnterminatedString);
        };

        // Calculate the byte position in the original `input` string
        let current_byte_pos_relative_to_original_input =
            input.len() - cur_input.len() + idx + c.len_utf8();

        match c {
            '"' => {
                return Ok((
                    Value::String(parsed_string),
                    &input[current_byte_pos_relative_to_original_input..],
                ));
            }
            '\\' => {
                let Some((_, escaped_char)) = chars.next() else {
                    return Err(Error::InvalidEscapeSequence(
                        "Invalid escape sequence: '\\' at end of string.".to_string(),
                    ));
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
                        let mut hex_val: u32 = 0;
                        for _ in 0..4 {
                            match chars.next() {
                                Some((_, c)) => {
                                    let digit =
                                        c.to_digit(16).ok_or(Error::InvalidUnicodeEscape)?;
                                    hex_val = (hex_val << 4) | digit;
                                }
                                None => {
                                    return Err(Error::InvalidUnicodeEscape);
                                }
                            }
                        }

                        let unicode_char =
                            char::from_u32(hex_val).ok_or(Error::InvalidUnicodeEscape)?;
                        parsed_string.push(unicode_char);
                    }
                    _ => {
                        return Err(Error::InvalidEscapeSequence(format!(
                            "Invalid escape sequence: '\\{}'",
                            escaped_char
                        )));
                    }
                }
            }
            _ if c == '\n' || c == '\r' || c == '\t' => {
                return Err(Error::UnexpectedToken(format!(
                    "Unescaped control character in string: '{}'",
                    c
                )));
            }
            _ => {
                parsed_string.push(c);
            }
        }
    }
}

fn number(input: &str) -> Result<ValueAndRest> {
    let mut cur_input = eat_whitespace(input);

    let mut minus = false;
    if let Some(rest) = cur_input.strip_prefix('-') {
        minus = true;
        cur_input = rest;
    }

    let mut buf = String::new();
    let mut enable_sign = false;
    for c in cur_input.chars() {
        match c {
            '0'..='9' => buf.push(c),
            '.' => buf.push(c),
            'e' | 'E' => {
                enable_sign = true;
                buf.push(c);
            }
            '-' | '+' => {
                if enable_sign {
                    buf.push(c);
                    enable_sign = false;
                } else {
                    return Err(Error::InvalidNumberFormat(
                        "sign only allowed at the beginning of the number or immediately after 'e' or 'E' for exponents".to_string(),
                    ));
                }
            }
            _ => break, // the char is not part of number.
        }
    }

    cur_input = cur_input.strip_prefix(&buf).unwrap();
    if minus {
        Ok((Value::Number(buf.parse::<f64>().unwrap() * -1.0), cur_input))
    } else {
        Ok((Value::Number(buf.parse::<f64>().unwrap()), cur_input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn parse_empty_object() {
        let json = "{}";
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
        match parsed {
            Value::String(s) => {
                assert_eq!(s, "hello \"world\"\\/\x08\x0c\x0a\x0d\tA");
            }
            _ => panic!("Expected a string, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_unterminated_string() {
        let json = r#""hello"#;
        let err = parse(json).unwrap_err();
        assert_eq!(err, Error::UnterminatedString);
    }

    #[test]
    fn parse_string_with_invalid_escape() {
        let json = r#""hello\x""#;
        let err = parse(json).unwrap_err();
        assert_eq!(
            err,
            Error::InvalidEscapeSequence("Invalid escape sequence: '\\x'".to_string())
        );
    }

    #[test]
    fn parse_string_with_incomplete_unicode_escape() {
        let json = r#""\u123""#;
        let err = parse(json).unwrap_err();
        assert_eq!(err, Error::InvalidUnicodeEscape);
    }

    #[test]
    fn parse_string_with_invalid_unicode_hex() {
        let json = r#""\u123G""#;
        let err = parse(json).unwrap_err();
        assert_eq!(err, Error::InvalidUnicodeEscape);
    }

    #[test]
    fn parse_string_with_valid_unicode_hex() {
        let json = r#""\u3042""#;
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
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
    fn parse_extra_characters_after_value() {
        let json = r#"{}extra"#;
        let err = parse(json).unwrap_err();
        assert_eq!(
            err,
            Error::TrailingCharacters(
                "Unexpected characters after JSON value: 'extra'".to_string()
            )
        );
    }

    #[test]
    fn parse_object_missing_colon() {
        let json = r#"{"key" "value"}"#;
        let err = parse(json).unwrap_err();
        assert_eq!(
            err,
            Error::MissingExpectedChar(':', " \"value\"}".to_string())
        );
    }

    #[test]
    fn parse_object_missing_comma_or_brace() {
        let json = r#"{"key": "value" "another_key": "another_value"}"#;
        let err = parse(json).unwrap_err();
        assert_eq!(
            err,
            Error::UnexpectedToken(
                "Expected ',' or '}' after object value. Found: ' \"another_key\": \"another_value\"}'"
                    .to_string()
            )
        );
    }

    #[test]
    fn parse_number() {
        let json = r#"10"#;
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
        match parsed {
            Value::Number(n) => {
                assert_eq!(n, 10000.0)
            }
            _ => panic!("Expected a number, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_number_with_minus_exponent() {
        let json = r#"10e-3"#;
        let parsed = parse(json).unwrap();
        match parsed {
            Value::Number(n) => {
                assert_eq!(n, 0.01)
            }
            _ => panic!("Expected a number, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_number_with_plus_exponent() {
        let json = r#"10e+3"#;
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
        match parsed {
            Value::Array(arr) => assert_eq!(arr, vec![]),
            _ => panic!("Expected an array, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_array_with_single_object() {
        let json = r#"[{"key1": true}]"#;
        let parsed = parse(json).unwrap();
        match parsed {
            Value::Array(arr) => assert_eq!(
                arr,
                vec![Value::Object(IndexMap::from([(
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
        let parsed = parse(json).unwrap();
        match parsed {
            Value::Array(arr) => assert_eq!(
                arr,
                vec![
                    Value::Object(IndexMap::from([("key1".to_string(), Value::Boolean(true))])),
                    Value::Object(IndexMap::from([("key1".to_string(), Value::Boolean(true))])),
                ]
            ),
            _ => panic!("Expected an array, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_array_with_single_array() {
        let json = r#"[[]]"#;
        let parsed = parse(json).unwrap();
        match parsed {
            Value::Array(arr) => assert_eq!(arr, vec![Value::Array(vec![])]),
            _ => panic!("Expected an array, got {:?}", parsed),
        }
    }

    #[test]
    fn parse_array_with_multiple_arrays() {
        let json = r#"[[],[],[]]"#;
        let parsed = parse(json).unwrap();
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
        let parsed = parse(json).unwrap();
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
        let v = parse(json).unwrap();
        assert_eq!(v["Image"]["Width"], "800.0".into());
        assert_eq!(v["Image"]["Height"], "600.0".into());
        assert_eq!(v["Image"]["Title"], r#""View from 15th Floor""#.into());
        assert_eq!(
            v["Image"]["Thumbnail"]["Url"],
            r#""http://www.example.com/image/481989943""#.into()
        );
        assert_eq!(v["Image"]["Thumbnail"]["Height"], "125.0".into());
        assert_eq!(v["Image"]["Thumbnail"]["Width"], "100.0".into());
        assert_eq!(v["Image"]["Animated"], "false".into());
        assert_eq!(v["Image"]["IDs"], "[116,943,234,38793]".into());
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
        let v = parse(json).unwrap();
        assert_eq!(v[0]["precision"], r#""zip""#.into());
        assert_eq!(v[0]["Latitude"], "37.7668".into());
        assert_eq!(v[0]["Longitude"], "-122.3959".into());
        assert_eq!(v[0]["Address"], r#""""#.into());
        assert_eq!(v[0]["City"], r#""SAN FRANCISCO""#.into());
        assert_eq!(v[0]["State"], r#""CA""#.into());
        assert_eq!(v[0]["Zip"], r#""94107""#.into());
        assert_eq!(v[0]["Country"], r#""US""#.into());
        assert_eq!(v[1]["precision"], r#""zip""#.into());
        assert_eq!(v[1]["Latitude"], "37.371991".into());
        assert_eq!(v[1]["Longitude"], "-122.026020".into());
        assert_eq!(v[1]["Address"], r#""""#.into());
        assert_eq!(v[1]["City"], r#""SUNNYVALE""#.into());
        assert_eq!(v[1]["State"], r#""CA""#.into());
        assert_eq!(v[1]["Zip"], r#""94085""#.into());
        assert_eq!(v[1]["Country"], r#""US""#.into());
    }
}
