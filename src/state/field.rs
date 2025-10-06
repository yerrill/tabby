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

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum FieldState {
    Unset,
    None,
    Bool,
    Int,
    Float,
    Str,
    BoolOrNone,
    IntOrNone,
    FloatOrNone,
    StrOrNone,
}

impl FieldState {
    fn match_unset(other: &Self) -> Self {
        *other
    }

    fn match_none(other: &Self) -> Self {
        match other {
            Self::Bool => Self::BoolOrNone,
            Self::Int => Self::IntOrNone,
            Self::Float => Self::FloatOrNone,
            Self::Str => Self::StrOrNone,
            _ => *other,
        }
    }

    fn match_bool(other: &Self) -> Self {
        match other {
            Self::Unset => Self::Bool,
            Self::None => Self::BoolOrNone,
            Self::Bool => Self::Bool,
            Self::Int => Self::Str,
            Self::Float => Self::Str,
            Self::Str => Self::Str,
            Self::BoolOrNone => Self::BoolOrNone,
            Self::IntOrNone => Self::StrOrNone,
            Self::FloatOrNone => Self::StrOrNone,
            Self::StrOrNone => Self::StrOrNone,
        }
    }

    fn match_int(other: &Self) -> Self {
        match other {
            Self::Unset => Self::Int,
            Self::None => Self::IntOrNone,
            Self::Bool => Self::Str,
            Self::Int => Self::Int,
            Self::Float => Self::Float,
            Self::Str => Self::Str,
            Self::BoolOrNone => Self::StrOrNone,
            Self::IntOrNone => Self::IntOrNone,
            Self::FloatOrNone => Self::FloatOrNone,
            Self::StrOrNone => Self::StrOrNone,
        }
    }

    fn match_float(other: &Self) -> Self {
        match other {
            Self::Unset => Self::Float,
            Self::None => Self::FloatOrNone,
            Self::Bool => Self::Str,
            Self::Int => Self::Float,
            Self::Float => Self::Float,
            Self::Str => Self::Str,
            Self::BoolOrNone => Self::StrOrNone,
            Self::IntOrNone => Self::FloatOrNone,
            Self::FloatOrNone => Self::FloatOrNone,
            Self::StrOrNone => Self::StrOrNone,
        }
    }

    fn match_str(other: &Self) -> Self {
        match other {
            Self::Unset => Self::Str,
            Self::None => Self::StrOrNone,
            Self::Bool => Self::Str,
            Self::Int => Self::Str,
            Self::Float => Self::Str,
            Self::Str => Self::Str,
            Self::BoolOrNone => Self::StrOrNone,
            Self::IntOrNone => Self::StrOrNone,
            Self::FloatOrNone => Self::StrOrNone,
            Self::StrOrNone => Self::StrOrNone,
        }
    }

    fn match_bool_none(other: &Self) -> Self {
        match other {
            Self::Unset => Self::BoolOrNone,
            Self::None => Self::BoolOrNone,
            Self::Bool => Self::BoolOrNone,
            Self::Int => Self::StrOrNone,
            Self::Float => Self::StrOrNone,
            Self::Str => Self::StrOrNone,
            Self::BoolOrNone => Self::BoolOrNone,
            Self::IntOrNone => Self::StrOrNone,
            Self::FloatOrNone => Self::StrOrNone,
            Self::StrOrNone => Self::StrOrNone,
        }
    }

    fn match_int_none(other: &Self) -> Self {
        match other {
            Self::Unset => Self::IntOrNone,
            Self::None => Self::IntOrNone,
            Self::Bool => Self::StrOrNone,
            Self::Int => Self::IntOrNone,
            Self::Float => Self::FloatOrNone,
            Self::Str => Self::StrOrNone,
            Self::BoolOrNone => Self::StrOrNone,
            Self::IntOrNone => Self::IntOrNone,
            Self::FloatOrNone => Self::FloatOrNone,
            Self::StrOrNone => Self::StrOrNone,
        }
    }

    fn match_float_none(other: &Self) -> Self {
        match other {
            Self::Unset => Self::FloatOrNone,
            Self::None => Self::FloatOrNone,
            Self::Bool => Self::StrOrNone,
            Self::Int => Self::FloatOrNone,
            Self::Float => Self::FloatOrNone,
            Self::Str => Self::StrOrNone,
            Self::BoolOrNone => Self::StrOrNone,
            Self::IntOrNone => Self::FloatOrNone,
            Self::FloatOrNone => Self::FloatOrNone,
            Self::StrOrNone => Self::StrOrNone,
        }
    }

    fn match_str_none(_: &Self) -> Self {
        Self::StrOrNone
    }

    pub fn change(self, it: Self) -> Self {
        match &self {
            Self::Unset => Self::match_unset(&it),
            Self::None => Self::match_none(&it),
            Self::Bool => Self::match_bool(&it),
            Self::Int => Self::match_int(&it),
            Self::Float => Self::match_float(&it),
            Self::Str => Self::match_str(&it),
            Self::BoolOrNone => Self::match_bool_none(&it),
            Self::IntOrNone => Self::match_int_none(&it),
            Self::FloatOrNone => Self::match_float_none(&it),
            Self::StrOrNone => Self::match_str_none(&it),
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Unset => "Unset",
            Self::None => "None",
            Self::Bool => "Bool",
            Self::Int => "Int",
            Self::Float => "Float",
            Self::Str => "Str",
            Self::BoolOrNone => "Bool | None",
            Self::IntOrNone => "Int | None",
            Self::FloatOrNone => "Float | None",
            Self::StrOrNone => "Str | None",
        }
    }

    pub fn from_str(data: &str) -> Self {
        const BOOLLIKE: [&'static str; 4] = ["true", "false", "yes", "no"];
        const NULLLIKE: [&'static str; 3] = ["null", "none", "nan"];

        let lower = data.to_lowercase();
        let data = data.trim();

        if data.len() == 0 || NULLLIKE.contains(&lower.as_str()) {
            return Self::None;
        }

        if BOOLLIKE.contains(&lower.as_str()) {
            return Self::Bool;
        }

        if check_integer(data) {
            return Self::Int;
        }

        if check_float(data) {
            return Self::Float;
        }

        Self::Str
    }
}

#[cfg(test)]
mod tests {
    use super::{FieldState as fs, *};

    fn apply_changes(changes: &[fs], expected: fs) {
        let mut field = fs::Unset;

        for &data in changes {
            field = field.change(data);
        }

        assert_eq!(expected, field);
    }

    #[test]
    fn state_matchine() {
        // Unset
        apply_changes(&[fs::None], fs::None);
        apply_changes(&[fs::Bool], fs::Bool);
        apply_changes(&[fs::Int], fs::Int);
        apply_changes(&[fs::Float], fs::Float);
        apply_changes(&[fs::Str], fs::Str);
        apply_changes(&[fs::BoolOrNone], fs::BoolOrNone);
        apply_changes(&[fs::IntOrNone], fs::IntOrNone);
        apply_changes(&[fs::FloatOrNone], fs::FloatOrNone);
        apply_changes(&[fs::StrOrNone], fs::StrOrNone);

        // None
        apply_changes(&[fs::None, fs::None], fs::None);
        apply_changes(&[fs::None, fs::Bool], fs::BoolOrNone);
        apply_changes(&[fs::None, fs::Int], fs::IntOrNone);
        apply_changes(&[fs::None, fs::Float], fs::FloatOrNone);
        apply_changes(&[fs::None, fs::Str], fs::StrOrNone);
        apply_changes(&[fs::None, fs::BoolOrNone], fs::BoolOrNone);
        apply_changes(&[fs::None, fs::IntOrNone], fs::IntOrNone);
        apply_changes(&[fs::None, fs::FloatOrNone], fs::FloatOrNone);
        apply_changes(&[fs::None, fs::StrOrNone], fs::StrOrNone);

        // Bool
        apply_changes(&[fs::Bool, fs::None], fs::BoolOrNone);
        apply_changes(&[fs::Bool, fs::Bool], fs::Bool);
        apply_changes(&[fs::Bool, fs::Int], fs::Str);
        apply_changes(&[fs::Bool, fs::Float], fs::Str);
        apply_changes(&[fs::Bool, fs::Str], fs::Str);
        apply_changes(&[fs::Bool, fs::BoolOrNone], fs::BoolOrNone);
        apply_changes(&[fs::Bool, fs::IntOrNone], fs::StrOrNone);
        apply_changes(&[fs::Bool, fs::FloatOrNone], fs::StrOrNone);
        apply_changes(&[fs::Bool, fs::StrOrNone], fs::StrOrNone);

        // Int
        apply_changes(&[fs::Int, fs::None], fs::IntOrNone);
        apply_changes(&[fs::Int, fs::Bool], fs::Str);
        apply_changes(&[fs::Int, fs::Int], fs::Int);
        apply_changes(&[fs::Int, fs::Float], fs::Float);
        apply_changes(&[fs::Int, fs::Str], fs::Str);
        apply_changes(&[fs::Int, fs::BoolOrNone], fs::StrOrNone);
        apply_changes(&[fs::Int, fs::IntOrNone], fs::IntOrNone);
        apply_changes(&[fs::Int, fs::FloatOrNone], fs::FloatOrNone);
        apply_changes(&[fs::Int, fs::StrOrNone], fs::StrOrNone);

        // Float
        apply_changes(&[fs::Float, fs::None], fs::FloatOrNone);
        apply_changes(&[fs::Float, fs::Bool], fs::Str);
        apply_changes(&[fs::Float, fs::Int], fs::Float);
        apply_changes(&[fs::Float, fs::Float], fs::Float);
        apply_changes(&[fs::Float, fs::Str], fs::Str);
        apply_changes(&[fs::Float, fs::BoolOrNone], fs::StrOrNone);
        apply_changes(&[fs::Float, fs::IntOrNone], fs::FloatOrNone);
        apply_changes(&[fs::Float, fs::FloatOrNone], fs::FloatOrNone);
        apply_changes(&[fs::Float, fs::StrOrNone], fs::StrOrNone);

        // Str
        apply_changes(&[fs::Str, fs::None], fs::StrOrNone);
        apply_changes(&[fs::Str, fs::Bool], fs::Str);
        apply_changes(&[fs::Str, fs::Int], fs::Str);
        apply_changes(&[fs::Str, fs::Float], fs::Str);
        apply_changes(&[fs::Str, fs::Str], fs::Str);
        apply_changes(&[fs::Str, fs::BoolOrNone], fs::StrOrNone);
        apply_changes(&[fs::Str, fs::IntOrNone], fs::StrOrNone);
        apply_changes(&[fs::Str, fs::FloatOrNone], fs::StrOrNone);
        apply_changes(&[fs::Str, fs::StrOrNone], fs::StrOrNone);

        // BoolOrNone
        apply_changes(&[fs::None, fs::Bool, fs::None], fs::BoolOrNone);
        apply_changes(&[fs::None, fs::Bool, fs::Bool], fs::BoolOrNone);
        apply_changes(&[fs::None, fs::Bool, fs::Int], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Bool, fs::Float], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Bool, fs::Str], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Bool, fs::BoolOrNone], fs::BoolOrNone);
        apply_changes(&[fs::None, fs::Bool, fs::IntOrNone], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Bool, fs::FloatOrNone], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Bool, fs::StrOrNone], fs::StrOrNone);

        // IntOrNone
        apply_changes(&[fs::None, fs::Int, fs::None], fs::IntOrNone);
        apply_changes(&[fs::None, fs::Int, fs::Bool], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Int, fs::Int], fs::IntOrNone);
        apply_changes(&[fs::None, fs::Int, fs::Float], fs::FloatOrNone);
        apply_changes(&[fs::None, fs::Int, fs::Str], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Int, fs::BoolOrNone], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Int, fs::IntOrNone], fs::IntOrNone);
        apply_changes(&[fs::None, fs::Int, fs::FloatOrNone], fs::FloatOrNone);
        apply_changes(&[fs::None, fs::Int, fs::StrOrNone], fs::StrOrNone);

        // FloatOrNone
        apply_changes(&[fs::None, fs::Float, fs::None], fs::FloatOrNone);
        apply_changes(&[fs::None, fs::Float, fs::Bool], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Float, fs::Int], fs::FloatOrNone);
        apply_changes(&[fs::None, fs::Float, fs::Float], fs::FloatOrNone);
        apply_changes(&[fs::None, fs::Float, fs::Str], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Float, fs::BoolOrNone], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Float, fs::IntOrNone], fs::FloatOrNone);
        apply_changes(&[fs::None, fs::Float, fs::FloatOrNone], fs::FloatOrNone);
        apply_changes(&[fs::None, fs::Float, fs::StrOrNone], fs::StrOrNone);

        // StrOrNone
        apply_changes(&[fs::None, fs::Str, fs::None], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Str, fs::Bool], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Str, fs::Int], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Str, fs::Float], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Str, fs::Str], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Str, fs::BoolOrNone], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Str, fs::IntOrNone], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Str, fs::FloatOrNone], fs::StrOrNone);
        apply_changes(&[fs::None, fs::Str, fs::StrOrNone], fs::StrOrNone);
    }

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
