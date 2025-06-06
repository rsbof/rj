// Defined in RFC8259 also known as STD90.

use value::Value;

mod generate;
mod parse;
mod value;

pub fn parse(input: &str) -> Value {
    parse::parse(input)
}

pub fn stringify(value: &Value) -> String {
    generate::generate(value)
}
