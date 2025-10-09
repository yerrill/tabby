use regex::Regex;

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

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum FieldState {
    None,
    Bool,
    Int,
    Float,
    Str,
}

impl FieldState {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Bool => "Bool",
            Self::Int => "Int",
            Self::Float => "Float",
            Self::Str => "Str",
        }
    }

    pub fn from_str(data: &str) -> Self {
        const BOOLLIKE: [&'static str; 4] = ["true", "false", "yes", "no"];
        const NULLLIKE: [&'static str; 3] = ["null", "none", "nan"];

        let data = data.trim().to_lowercase();

        if data.len() == 0 || NULLLIKE.contains(&data.as_str()) {
            return Self::None;
        }

        if BOOLLIKE.contains(&data.as_str()) {
            return Self::Bool;
        }

        if check_integer(data.as_str()) {
            return Self::Int;
        }

        if check_float(data.as_str()) {
            return Self::Float;
        }

        Self::Str
    }
}

#[cfg(test)]
mod tests {
    use super::{FieldState as fs, *};

    #[test]
    fn data_type_parsing() {
        const NONES: [&'static str; 2] = ["   ", ""];

        for test in NONES {
            assert_eq!(fs::from_str(test), fs::None);
        }

        const BOOLS: [&'static str; 12] = [
            "true", "True", "TRUE", "tRuE", "false", "False", "FALSE", "FaLsE", "  yes  ", "no  ",
            "YES", "NO",
        ];

        for test in BOOLS {
            println!("{}", test);
            assert_eq!(fs::from_str(test), fs::Bool);
        }

        const INTS: [&'static str; 13] = [
            "1",
            "2222",
            "1 000 000",
            "-$1 000 000",
            "33",
            "0",
            "-0",
            "-2",
            "$1321",
            "-$123",
            "$-001",
            "($123)",
            "$(123)",
        ];

        for test in INTS {
            assert!(check_integer(test));
            assert_eq!(fs::from_str(test), fs::Int);
        }

        const FLOATS: [&'static str; 6] =
            ["1.1", "213.001", "-234.5", "$2.2", "$-123.21", "-$12.321"];

        for test in FLOATS {
            assert!(check_float(test));
            assert_eq!(fs::from_str(test), fs::Float);
        }

        for test in INTS {
            assert!(!check_float(test));
        }

        for test in FLOATS {
            assert!(!check_integer(test));
        }

        const STRS: [&'static str; 5] = ["   yesa", "NN", "123a", "122.1x", "x 1 x 2"];

        for test in STRS {
            assert_eq!(fs::from_str(test), fs::Str);
        }
    }
}
