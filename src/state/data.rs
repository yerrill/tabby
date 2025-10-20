use regex::Regex;
use serde_json::{Number, Value};
use std::collections::HashMap;

const REMOVE_CHARS_PRE: [char; 2] = [' ', '$'];
const REMOVE_CHARS_POST: [char; 3] = ['(', ')', '-'];

const PATTERNS_INTEGER_POSITIVE: [&'static str; 1] = [r"^[[:digit:]]+$"];
const PATTERNS_INTEGER_NEGATIVE: [&'static str; 2] = [r"^-[[:digit:]]+$", r"^\([[:digit:]]+\)$"];

const PATTERNS_FLOAT_POSITIVE: [&'static str; 1] = [r"^[[:digit:]]+\.[[:digit:]]+$"];
const PATTERNS_FLOAT_NEGATIVE: [&'static str; 2] = [
    r"^-[[:digit:]]+\.[[:digit:]]+$",
    r"^\([[:digit:]]+\.[[:digit:]]+\)$",
];

fn check_integer(data: &str) -> Option<i64> {
    let data = data.replace(&REMOVE_CHARS_PRE, "");
    dbg!(&data);

    for pattern in PATTERNS_INTEGER_POSITIVE {
        let re = Regex::new(pattern).unwrap();

        if re.is_match(data.as_str()) {
            let cleaned = data.replace(&REMOVE_CHARS_POST, "");
            return Some(
                cleaned
                    .parse::<i64>()
                    .expect(format!("Unable to parse integer value {:?}", data).as_str()),
            );
        }
    }

    for pattern in PATTERNS_INTEGER_NEGATIVE {
        let re = Regex::new(pattern).unwrap();

        if re.is_match(data.as_str()) {
            let cleaned = data.replace(&REMOVE_CHARS_POST, "");
            return Some(
                cleaned
                    .parse::<i64>()
                    .expect(format!("Unable to parse integer value {:?}", data).as_str())
                    * -1,
            );
        }
    }

    None
}

fn check_float(data: &str) -> Option<f64> {
    let data = data.replace(&REMOVE_CHARS_PRE, "");
    dbg!(&data);

    for pattern in PATTERNS_FLOAT_POSITIVE {
        let re = Regex::new(pattern).unwrap();

        if re.is_match(data.as_str()) {
            let cleaned = data.replace(&REMOVE_CHARS_POST, "");
            return Some(
                cleaned
                    .parse::<f64>()
                    .expect(format!("Unable to parse integer value {:?}", data).as_str()),
            );
        }
    }

    for pattern in PATTERNS_FLOAT_NEGATIVE {
        let re = Regex::new(pattern).unwrap();

        if re.is_match(data.as_str()) {
            let cleaned = data.replace(&REMOVE_CHARS_POST, "");
            return Some(
                cleaned
                    .parse::<f64>()
                    .expect(format!("Unable to parse integer value {:?}", data).as_str())
                    * -1_f64,
            );
        }
    }

    None
}

fn refine_number(num: Number) -> Literals {
    if num.is_f64() {
        Literals::Float(num.as_f64().unwrap().to_bits())
    } else {
        Literals::Integer(num.as_i64().unwrap())
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Literals {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(u64),
    String(String),
}

impl From<&str> for Literals {
    fn from(data: &str) -> Self {
        const TRUTHY: [&'static str; 2] = ["true", "yes"];
        const FALSEY: [&'static str; 2] = ["false", "no"];
        const NULLLIKE: [&'static str; 3] = ["null", "none", "nan"];

        let cleaned = data.trim().to_lowercase();

        if cleaned.len() == 0 || NULLLIKE.contains(&cleaned.as_str()) {
            return Self::Null;
        }

        if TRUTHY.contains(&cleaned.as_str()) {
            return Self::Boolean(true);
        }

        if FALSEY.contains(&cleaned.as_str()) {
            return Self::Boolean(false);
        }

        if let Some(i) = check_integer(cleaned.as_str()) {
            return Self::Integer(i);
        }

        if let Some(f) = check_float(cleaned.as_str()) {
            return Self::Float(f.to_bits());
        }

        Self::String(data.to_owned())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataValues {
    Literal(Literals),
    Array(Vec<DataValues>),
    Object(HashMap<String, DataValues>),
}

impl From<Value> for DataValues {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => Self::Literal(Literals::Null),
            Value::Bool(b) => Self::Literal(Literals::Boolean(b)),
            Value::Number(n) => Self::Literal(refine_number(n)),
            Value::String(s) => Self::Literal(Literals::String(s)),
            Value::Array(a) => Self::Array(a.into_iter().map(|v| Self::from(v)).collect()),
            Value::Object(m) => {
                Self::Object(m.into_iter().map(|(k, v)| (k, Self::from(v))).collect())
            }
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
    /// Encoding number interpretation expectations
    fn serde_numbers() {
        let n1 = serde_json::Number::from_i128(12345).unwrap();
        let n2 = serde_json::Number::from_f64(12345.1).unwrap();
        let n3 = serde_json::Number::from_f64(12345.0).unwrap();

        assert!(n1.is_i64());
        assert!(n2.is_f64());
        assert!(n3.is_f64());

        assert!(!n1.is_f64());
        assert!(!n2.is_i64());
        assert!(!n3.is_i64());

        assert!(refine_number(n1) == Literals::Integer(12345));
        assert!(refine_number(n2) == Literals::Float((12345.1_f64).to_bits()));
        assert!(refine_number(n3) == Literals::Float((12345.0_f64).to_bits()));
    }

    #[test]
    fn data_type_parsing() {
        const NULLS: [&'static str; 4] = ["   ", "", "null", "none"];

        for test in NULLS {
            assert_eq!(Literals::from(test), Literals::Null);
        }

        const TRUES: [&'static str; 6] = ["true", "True", "TRUE", "tRuE", "  yes  ", "YES"];

        for test in TRUES {
            assert_eq!(Literals::from(test), Literals::Boolean(true));
        }

        const FALSES: [&'static str; 6] = ["false", "False", "FALSE", "FaLsE", "NO", "no  "];

        for test in FALSES {
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
            ("-$123", -123),
            ("$-001", -1),
            ("($123)", -123),
            ("$(123)", -123),
        ];

        for (input, result) in INTS {
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
            assert_eq!(Literals::from(input), Literals::Float(value.to_bits()));
        }

        for (test, _) in INTS {
            assert!(check_float(test).is_none());
        }

        for (test, _) in FLOATS {
            assert!(check_integer(test).is_none());
        }

        const STRS: [&'static str; 5] = ["   yesa", "NN", "123a", "122.1x", "x 1 x 2"];

        for test in STRS {
            assert_eq!(Literals::from(test), Literals::String(test.to_owned()));
        }
    }
}
