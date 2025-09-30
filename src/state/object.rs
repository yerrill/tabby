use super::FieldState;
use std::collections::HashMap;

fn tabs(count: usize) -> String {
    (0..count).map(|_| "  ").collect()
}

#[derive(PartialEq, Eq, Debug)]
pub enum StateObject {
    Type(FieldState),
    Array(Vec<StateObject>),
    Object(HashMap<String, StateObject>),
}

impl StateObject {
    pub fn to_str(&self, indent: usize) -> String {
        let mut output = String::new();

        match self {
            Self::Type(f) => {
                output += f.to_str();
            }
            Self::Array(a) => {
                output += "[\n";

                let mut list = Vec::new();

                for value in a.iter() {
                    list.push(value.to_str(indent + 1));
                }

                output += &list
                    .into_iter()
                    .map(|i| tabs(indent + 1) + &i)
                    .collect::<Vec<_>>()
                    .join(",\n");

                output += "\n";
                output += &tabs(indent);
                output += "]";
            }
            Self::Object(o) => {
                output += "{\n";

                let mut obj = Vec::new();

                for (key, value) in o.iter() {
                    obj.push(String::new() + key + ": " + &value.to_str(indent + 1));
                }

                output += &obj
                    .into_iter()
                    .map(|i| tabs(indent + 1) + &i)
                    .collect::<Vec<_>>()
                    .join(",\n");

                output += "\n";
                output += &tabs(indent);
                output += "}";
            }
        };

        output
    }
}
