use super::{CodegenOptions, Generation};
use crate::state::{Literals, ObjectProperty, Subschema};
use serde_json::{Value, json, to_string_pretty};
use std::collections::{HashMap, HashSet};

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
    fn from_literal(field: &Literals) -> Self {
        match field {
            Literals::Null => Self::Null,
            Literals::Boolean(_) => Self::Boolean,
            Literals::Integer(_) => Self::Integer,
            Literals::Float(_) => Self::Number,
            Literals::String(_) => Self::String,
        }
    }

    fn to_string(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::String => "string",
        }
    }
}

fn literal_to_value(l: Literals) -> Value {
    match l {
        Literals::Null => Value::Null,
        Literals::Boolean(b) => Value::Bool(b),
        Literals::Integer(i) => Value::Number(i.into()),
        Literals::Float(f) => Value::Number(f.into()),
        Literals::String(s) => Value::String(s),
    }
}

fn literals_to_json(
    types: HashSet<Literals>,
    types_instance_count: usize,
    options: &CodegenOptions,
) -> Option<Value> {
    let mut primatives = types
        .iter()
        .map(|i| TypePrimative::from_literal(i).to_string())
        .collect::<HashSet<_>>()
        .into_iter();

    let type_part = if primatives.len() == 1 {
        json!(primatives.next().unwrap())
    } else if primatives.len() > 1 {
        json!(primatives.collect::<Vec<_>>())
    } else {
        return None;
    };

    // If unique values are less than total count * ratio
    if types.len() < (types_instance_count * options.enum_threshold as usize) / 100 {
        dbg!("triggered");
        Some(
            json!({"types": type_part, "enum": types.into_iter().map(|l| literal_to_value(l)).collect::<Vec<_>>()}),
        )
    } else {
        Some(json!({"type": type_part}))
    }
}

fn subschema_to_json(
    Subschema {
        types,
        array,
        object,
        types_instance_count,
    }: Subschema,
    options: &CodegenOptions,
) -> Value {
    let mut schemas: Vec<Value> = Vec::new();

    // Terminal cases

    if let Some(s) = literals_to_json(types, types_instance_count, options) {
        schemas.push(s);
    }

    // Array case
    if let Some(a) = array {
        schemas.push(json!({"type": "array", "items": subschema_to_json(*a, options)}));
    };

    // Object case
    if let Some(o) = object {
        let required = o
            .iter()
            .filter(|(_, ObjectProperty { value: _, required })| *required)
            .map(|(k, _)| k.to_owned())
            .collect::<Vec<_>>();

        let properties = o
            .into_iter()
            .map(
                |(
                    k,
                    ObjectProperty {
                        value: v,
                        required: _,
                    },
                )| (k, subschema_to_json(v, options)),
            )
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
    fn generate(sb: Subschema, options: CodegenOptions) -> String {
        let values = subschema_to_json(sb, &options);
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
