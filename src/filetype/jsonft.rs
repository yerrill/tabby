use super::Filetype;
use crate::state::DataValues;
use serde_json::Value;

pub struct JsonFileType {
    json: Value,
}

impl JsonFileType {
    pub fn new(file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = serde_json::from_str(file)?;

        Ok(Self { json })
    }
}

impl Filetype for JsonFileType {
    fn to_object(self) -> DataValues {
        DataValues::from(self.json)
    }
}

#[cfg(test)]
mod tests {
    use super::super::Filetype;
    use super::JsonFileType as jft;
    use crate::state::{DataValues as DV, Literals as LT};
    use std::collections::HashMap;

    #[test]
    fn json() {
        let none = jft::new("null").unwrap();
        assert_eq!(none.to_object(), DV::Literal(LT::Null));

        let bool = jft::new("true").unwrap();
        assert_eq!(bool.to_object(), DV::Literal(LT::Boolean(true)));

        let int = jft::new("-1").unwrap();
        assert_eq!(int.to_object(), DV::Literal(LT::Integer(-1)));

        let float = jft::new("1.2").unwrap();
        assert_eq!(
            float.to_object(),
            DV::Literal(LT::Float((1.2_f64).to_bits()))
        );

        let obj_empty = jft::new("{}").unwrap();
        assert_eq!(obj_empty.to_object(), DV::Object(HashMap::new()));

        let obj_basic = jft::new("{\"a\": 1, \"b\": \"abc\"}").unwrap();
        assert_eq!(
            obj_basic.to_object(),
            DV::Object(
                vec![
                    (String::from("a"), DV::Literal(LT::Integer(1))),
                    (
                        String::from("b"),
                        DV::Literal(LT::String(String::from("abc")))
                    )
                ]
                .into_iter()
                .collect()
            )
        );

        let list_empty = jft::new("[]").unwrap();
        assert_eq!(list_empty.to_object(), DV::Array(Vec::new()));

        let list_basic = jft::new("[1, 3.2, true, {}, []]").unwrap();
        assert_eq!(
            list_basic.to_object(),
            DV::Array(vec![
                DV::Literal(LT::Integer(1)),
                DV::Literal(LT::Float((3.2_f64).to_bits())),
                DV::Literal(LT::Boolean(true)),
                DV::Object(HashMap::new()),
                DV::Array(Vec::new())
            ])
        );

        let advanced = jft::new(
            "{\"a\": 1, \"b\": [1, true, {\"c\": [1.2, 1.3, 1.4]}], \"d\": [\"x\", \"y\", \"z\"]}",
        )
        .unwrap();
        assert_eq!(
            advanced.to_object(),
            DV::Object(
                vec![
                    (String::from("a"), DV::Literal(LT::Integer(1))),
                    (
                        String::from("b"),
                        DV::Array(vec![
                            DV::Literal(LT::Integer(1)),
                            DV::Literal(LT::Boolean(true)),
                            DV::Object(
                                vec![(
                                    String::from("c"),
                                    DV::Array(vec![
                                        DV::Literal(LT::Float((1.2_f64).to_bits())),
                                        DV::Literal(LT::Float((1.3_f64).to_bits())),
                                        DV::Literal(LT::Float((1.4_f64).to_bits()))
                                    ])
                                )]
                                .into_iter()
                                .collect()
                            )
                        ])
                    ),
                    (
                        String::from("d"),
                        DV::Array(vec![
                            DV::Literal(LT::String(String::from("x"))),
                            DV::Literal(LT::String(String::from("y"))),
                            DV::Literal(LT::String(String::from("z")))
                        ])
                    )
                ]
                .into_iter()
                .collect()
            )
        );
    }
}
