// Defined in RFC8259 also known as STD90.

use value::Value;

mod generate;
pub mod parse;
mod value;

pub fn parse(input: &str) -> Result<Value, parse::Error> {
    parse::parse(input)
}

pub fn stringify(value: &Value) -> String {
    value.to_string()
}

pub fn format(input: &str) -> Result<String, parse::Error> {
    Ok(generate::format(&parse(input)?, 2))
}
