use super::Generation;
use crate::state::{FieldState, StateObject, UnionObject};
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
}

enum Subschema {
    Terminal {
        primative_type: TypePrimative,
    },

    Array {
        items: Box<Subschema>,
    },

    Object {
        properties: HashMap<String, Subschema>,
    },
}

impl Subschema {
    fn from_union(uo: UnionObject) -> Self {
        let subschemas: Vec<Subschema> = Vec::new();

        // Terminal cases
        subschemas.extend(match uo.terminal {
            Some(t) => TypePrimative::from_field_state(t)
                .into_iter()
                .map(|p| Subschema::Terminal { primative_type: p })
                .collect(),
            None => Vec::new(),
        });

        if let Some(a) = uo.array {
            subschemas.push(Subschema::Array {
                items: Box::new(Self::from_union(*a)),
            });
        };

        if let Some(o) = uo.object {
            subschemas.push(Subschema::Object {
                properties: o
                    .into_iter()
                    .map(|(k, v)| (k, Subschema::from_union(v)))
                    .collect(),
            });
        };
    }
}

pub struct JsonSchema {}

impl Generation for JsonSchema {
    fn generate(object: StateObject) -> String {
        let uo = UnionObject::from_state_object(object);
        let subschemas = Subschema::from_union(uo);

        // Array cases
    }
}
