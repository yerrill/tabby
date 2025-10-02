use super::Filetype;
use crate::state::{FieldState, StateObject};
use serde_json::Value as sj_value;

pub struct JsonFileType {
    json: sj_value,
}

impl JsonFileType {
    pub fn new(file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json: sj_value = serde_json::from_str(file)?;

        Ok(Self { json })
    }

    fn refine_number(num: &serde_json::Number) -> FieldState {
        if num.is_f64() {
            FieldState::Float
        } else {
            FieldState::Int
        }
    }

    fn to_state_object(field: sj_value) -> StateObject {
        match field {
            sj_value::Null => StateObject::Type(FieldState::None),
            sj_value::Bool(_) => StateObject::Type(FieldState::Bool),
            sj_value::Number(n) => StateObject::Type(Self::refine_number(&n)),
            sj_value::String(_) => StateObject::Type(FieldState::Str),
            sj_value::Array(a) => {
                let mut array = Vec::new();

                for value in a {
                    array.push(Self::to_state_object(value));
                }

                StateObject::Array(array)
            }
            sj_value::Object(o) => StateObject::Object(
                o.into_iter()
                    .map(|(key, value)| (key, Self::to_state_object(value)))
                    .collect(),
            ),
        }
    }
}

impl Filetype for JsonFileType {
    fn to_object(self) -> StateObject {
        Self::to_state_object(self.json)
    }
}

#[cfg(test)]
mod tests {
    use super::{JsonFileType as jft, *};
    use std::collections::HashMap;

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

        assert!(JsonFileType::refine_number(&n1) == FieldState::Int);
        assert!(JsonFileType::refine_number(&n2) == FieldState::Float);
        assert!(JsonFileType::refine_number(&n3) == FieldState::Float);
    }

    #[test]
    fn json() {
        let none = jft::new("null").unwrap();
        assert_eq!(
            jft::to_state_object(none.json),
            StateObject::Type(FieldState::None)
        );

        let bool = jft::new("true").unwrap();
        assert_eq!(
            jft::to_state_object(bool.json),
            StateObject::Type(FieldState::Bool)
        );

        let int = jft::new("-1").unwrap();
        assert_eq!(
            jft::to_state_object(int.json),
            StateObject::Type(FieldState::Int)
        );

        let float = jft::new("1.2").unwrap();
        assert_eq!(
            jft::to_state_object(float.json),
            StateObject::Type(FieldState::Float)
        );

        let obj_empty = jft::new("{}").unwrap();
        assert_eq!(
            jft::to_state_object(obj_empty.json),
            StateObject::Object(HashMap::new())
        );

        let obj_basic = jft::new("{\"a\": 1, \"b\": \"abc\"}").unwrap();
        assert_eq!(
            jft::to_state_object(obj_basic.json),
            StateObject::Object(
                vec![
                    (String::from("a"), StateObject::Type(FieldState::Int)),
                    (String::from("b"), StateObject::Type(FieldState::Str))
                ]
                .into_iter()
                .collect()
            )
        );

        let list_empty = jft::new("[]").unwrap();
        assert_eq!(
            jft::to_state_object(list_empty.json),
            StateObject::Array(Vec::new())
        );

        let list_basic = jft::new("[1, 3.2, true, {}, []]").unwrap();
        assert_eq!(
            jft::to_state_object(list_basic.json),
            StateObject::Array(vec![
                StateObject::Type(FieldState::Int),
                StateObject::Type(FieldState::Float),
                StateObject::Type(FieldState::Bool),
                StateObject::Object(HashMap::new()),
                StateObject::Array(Vec::new())
            ])
        );

        let advanced = jft::new(
            "{\"a\": 1, \"b\": [1, true, {\"c\": [1.2, 1.3, 1.4]}], \"d\": [\"x\", \"y\", \"z\"]}",
        )
        .unwrap();
        assert_eq!(
            jft::to_state_object(advanced.json),
            StateObject::Object(
                vec![
                    (String::from("a"), StateObject::Type(FieldState::Int)),
                    (
                        String::from("b"),
                        StateObject::Array(vec![
                            StateObject::Type(FieldState::Int),
                            StateObject::Type(FieldState::Bool),
                            StateObject::Object(
                                vec![(
                                    String::from("c"),
                                    StateObject::Array(vec![
                                        StateObject::Type(FieldState::Float),
                                        StateObject::Type(FieldState::Float),
                                        StateObject::Type(FieldState::Float)
                                    ])
                                )]
                                .into_iter()
                                .collect()
                            )
                        ])
                    ),
                    (
                        String::from("d"),
                        StateObject::Array(vec![
                            StateObject::Type(FieldState::Str),
                            StateObject::Type(FieldState::Str),
                            StateObject::Type(FieldState::Str)
                        ])
                    )
                ]
                .into_iter()
                .collect()
            )
        );
    }
}
