use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

const REMOVE_CHARS: [char; 2] = [' ', '$'];

const PATTERNS_INTEGER: [&'static str; 2] = [r"^-?[[:digit:]]+$", r"^\([[:digit:]]+\)$"];

const PATTERNS_FLOAT: [&'static str; 2] = [
    r"^-?[[:digit:]]+\.[[:digit:]]+$",
    r"^\([[:digit:]]+\.[[:digit:]]+\)$",
];

fn check_integer(data: &str) -> bool {
    let data = data.replace(&REMOVE_CHARS, "");

    let mut result = false;

    for pattern in PATTERNS_INTEGER {
        let re = Regex::new(pattern).unwrap();

        result |= re.is_match(data.as_str());
    }

    result
}

fn check_float(data: &str) -> bool {
    let data = data.replace(&REMOVE_CHARS, "");

    let mut result = false;

    for pattern in PATTERNS_FLOAT {
        let re = Regex::new(pattern).unwrap();

        result |= re.is_match(data.as_str());
    }

    result
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Literals {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(u64),
    String(String),
}

impl Literals {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Boolean(_) => "boolean",
            Self::Integer(_) => "integer",
            Self::Float(_) => "number",
            Self::String(_) => "string",
        }
    }
}

impl From<&str> for Literals {
    fn from(data: &str) -> Self {
        const TRUTHY: [&'static str; 2] = ["true", "yes"];
        const FALSEY: [&'static str; 2] = ["false", "no"];
        const NULLLIKE: [&'static str; 3] = ["null", "none", "nan"];

        let data = data.trim().to_lowercase();

        if data.len() == 0 || NULLLIKE.contains(&data.as_str()) {
            return Self::Null;
        }

        if TRUTHY.contains(&data.as_str()) {
            return Self::Boolean(true);
        }

        if FALSEY.contains(&data.as_str()) {
            return Self::Boolean(false);
        }

        if check_integer(data.as_str()) {
            return Self::Integer(
                data.parse()
                    .expect(format!("Unable to parse integer value {:?}", data).as_str()),
            );
        }

        if check_float(data.as_str()) {
            return Self::Float(
                data.parse::<f64>()
                    .expect(format!("Unable to parse float value {:?}", data).as_str())
                    .to_bits(),
            );
        }

        Self::String(data.to_owned())
    }
}

#[derive(Clone, Debug)]
pub enum DataValues {
    Literal(Literals),
    Array(Vec<DataValues>),
    Object(HashMap<String, DataValues>),
}

impl From<Value> for DataValues {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => DataValues::Literal(Literals::Null),
            Value::Bool(b) => DataValues::Literal(Literals::Boolean(b)),
            Value::Number(number) => todo!(),
            Value::String(_) => todo!(),
            Value::Array(values) => todo!(),
            Value::Object(map) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_f64() {
        assert_eq!(f64::from_bits((1f64).to_bits()), 1f64);
    }

    #[test]
    fn data_type_parsing() {
        const NONES: [&'static str; 2] = ["   ", ""];

        for test in NONES {
            assert_eq!(Literals::from(test), Literals::Null);
        }

        const TRUES: [&'static str; 6] = ["true", "True", "TRUE", "tRuE", "  yes  ", "YES"];

        for test in TRUES {
            println!("{}", test);
            assert_eq!(Literals::from(test), Literals::Boolean(true));
        }

        const FALSES: [&'static str; 6] = ["false", "False", "FALSE", "FaLsE", "NO", "no  "];

        for test in FALSES {
            println!("{}", test);
            assert_eq!(Literals::from(test), Literals::Boolean(false));
        }

        const INTS: [(&'static str, i64); 13] = [
            ("1", 1),
            ("2222", 2222),
            ("1 000 000", 1_000_000),
            ("-$1 000 000", -1_000_000),
            ("33", 33),
            ("0", 0),
            ("-0", 0),
            ("-2", -2),
            ("$1321", 1321),
            ("-$123", 123),
            ("$-001", -1),
            ("($123)", -123),
            ("$(123)", -123),
        ];

        for (input, result) in INTS {
            assert!(check_integer(input));
            assert_eq!(Literals::from(input), Literals::Integer(result));
        }

        const FLOATS: [(&'static str, f64); 6] = [
            ("1.1", 1.1),
            ("213.001", 213.001),
            ("-234.5", -234.5),
            ("$2.2", 2.2),
            ("$-123.21", -123.21),
            ("-$12.321", -12.321),
        ];

        for (input, value) in FLOATS {
            assert!(check_float(input));
            assert_eq!(Literals::from(input), Literals::Float(value.to_bits()));
        }

        for (test, _) in INTS {
            assert!(!check_float(test));
        }

        for (test, _) in FLOATS {
            assert!(!check_integer(test));
        }

        const STRS: [&'static str; 5] = ["   yesa", "NN", "123a", "122.1x", "x 1 x 2"];

        for test in STRS {
            assert_eq!(Literals::from(test), Literals::String(test.to_owned()));
        }
    }
}
