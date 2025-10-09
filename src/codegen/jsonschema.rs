use super::Generation;
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
    fn from_field_state(field: FieldState) -> Vec<TypePrimative> {
        match field {
            FieldState::Unset => vec![Self::Null],
            FieldState::None => vec![Self::Null],
            FieldState::Bool => vec![Self::Boolean],
            FieldState::Int => vec![Self::Integer],
            FieldState::Float => vec![Self::Number],
            FieldState::Str => vec![Self::String],
            FieldState::BoolOrNone => vec![Self::Boolean, Self::Null],
            FieldState::IntOrNone => vec![Self::Integer, Self::Null],
            FieldState::FloatOrNone => vec![Self::Number, Self::Null],
            FieldState::StrOrNone => vec![Self::String, Self::Null],
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
    for t in uo.terminal {
        let primatives = TypePrimative::from_field_state(t)
            .into_iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>();

        if primatives.len() > 1 {
            schemas.push(json!({"type": primatives}));
        } else if primatives.len() == 1 {
            schemas.push(json!({"type": primatives[0]}));
        } else {
            panic!("No primatives returnd for {:?}", t);
        }
    }

    // Array case
    if let Some(a) = uo.array {
        schemas.push(json!({"type": "array", "items": union_to_json(*a)}));
    };

    // Object case
    if let Some(o) = uo.object {
        schemas.push(json!({"type": "object", "properties":
            o.into_iter().map(|(k, v)| (k, union_to_json(v))).collect::<HashMap<_, _>>()
        }));
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
    fn generate(object: StateObject) -> String {
        let uo = UnionObject::from_state_object(object);

        let values = union_to_json(uo);
        let values = if let Value::Object(mut o) = values {
            let _ = o.insert(
                String::from("$schema"),
                Value::String(String::from(SCHEMA_VERSION)),
            );
            Value::Object(o)
        } else {
            values
        };

        to_string_pretty(&values)
            .expect(format!("Values unable to be printed {:?}", values).as_str())
    }
}
