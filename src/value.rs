use indexmap::IndexMap;
use std::{ops::Index, str::FromStr};

use crate::parse;

#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(IndexMap<String, Value>),
    Array(Vec<Value>),
}

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        if let Self::Object(obj) = self {
            &obj[index]
        } else {
            panic!("&str index only allowed for Value::Object");
        }
    }
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        if let Self::Array(arr) = self {
            &arr[index]
        } else {
            panic!("integer index only allowed for Value::Array");
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        crate::parse(value).unwrap()
    }
}

impl FromStr for Value {
    type Err = parse::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse(s)
    }
}
