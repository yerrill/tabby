use super::{CodegenOptions, Generation};
use crate::state::{FieldState, StateObject, UnionObject};
use serde_json::{Value, json, to_string_pretty};
use std::collections::HashMap;

const SCHEMA_VERSION: &'static str = "https://json-schema.org/draft/2020-12/schema";

#[derive(Clone, Copy)]
enum TypePrimative {
    Null,
    Boolean,
    // Object,
    // Array,
    Integer, // Doesn't appear to be part of core but an accepted vocabulary
    Number,
    String,
}

impl TypePrimative {
    fn from_field_state(field: FieldState) -> Self {
        match field {
            FieldState::None => Self::Null,
            FieldState::Bool => Self::Boolean,
            FieldState::Int => Self::Integer,
            FieldState::Float => Self::Number,
            FieldState::Str => Self::String,
        }
    }

    fn to_string(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::String => "string",
        }
    }
}

fn union_to_json(uo: UnionObject) -> Value {
    let mut schemas: Vec<Value> = Vec::new();

    // Terminal cases

    let mut primatives = uo
        .terminal
        .into_iter()
        .map(|i| TypePrimative::from_field_state(i).to_string());

    if primatives.len() == 1 {
        schemas.push(json!({"type": primatives.next().unwrap()}));
    } else if primatives.len() > 1 {
        schemas.push(json!({"type": primatives.collect::<Vec<_>>()}));
    }

    // Array case
    if let Some(a) = uo.array {
        schemas.push(json!({"type": "array", "items": union_to_json(*a)}));
    };

    // Object case
    if let Some(o) = uo.object {
        let required = o
            .iter()
            .filter(|(_, v)| !v.terminal.contains(&FieldState::None))
            .map(|(k, _)| k.to_owned())
            .collect::<Vec<_>>();

        let properties = o
            .into_iter()
            .map(|(k, v)| (k, union_to_json(v)))
            .collect::<HashMap<_, _>>();

        schemas.push(json!({"type": "object", "properties": properties, "required": required}));
    };

    if schemas.len() == 0 {
        json!({})
    } else if schemas.len() == 1 {
        schemas.pop().expect("Could not pop from schemas array")
    } else {
        json!({"anyOf": schemas})
    }
}

pub struct JsonSchema {}

impl Generation for JsonSchema {
    fn generate(object: StateObject, options: CodegenOptions) -> String {
        let uo = UnionObject::from_state_object(object);

        let values = union_to_json(uo);
        let values = if let Value::Object(mut o) = values {
            let _ = o.insert(
                String::from("$schema"),
                Value::String(String::from(SCHEMA_VERSION)),
            );

            if let Some(title) = &options.title {
                let _ = o.insert(String::from("title"), Value::String(title.clone()));
            }
            Value::Object(o)
        } else {
            values
        };

        to_string_pretty(&values)
            .expect(format!("Values unable to be printed {:?}", values).as_str())
    }
}
