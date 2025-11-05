use super::{CodegenOptions, Generation};
use crate::state::{Literals, ObjectProperty, Subschema, SubschemaTypes};
use serde_json::{Number, Value, json, to_string_pretty};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

const SCHEMA_VERSION: &str = "https://json-schema.org/draft/2020-12/schema";

#[derive(Clone, Copy)]
enum TypePrimative {
    Null,
    Boolean,
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

fn literal_to_value(l: Literals) -> Value {
    match l {
        Literals::Null => Value::Null,
        Literals::Boolean(b) => Value::Bool(b),
        Literals::Integer(i) => Value::Number(i.into()),
        Literals::Float(f) => Value::from(Number::from_f64(f64::from_bits(f))),
        Literals::String(s) => Value::String(s),
    }
}

fn literals_to_json(types: SubschemaTypes, options: &CodegenOptions) -> Value {
    let mut primatives = types
        .values
        .iter()
        .map(|i| TypePrimative::from_literal(i).to_string())
        .collect::<HashSet<_>>()
        .into_iter();

    let type_part = match primatives.len().cmp(&1) {
        Ordering::Equal => json!(primatives.next().unwrap()),
        Ordering::Greater => json!(primatives.collect::<Vec<_>>()),
        _ => panic!("(Unreachable) Collapsing subscheama types with no values"),
    };

    // If unique values are less than total count * ratio
    let unique_threshold =
        types.values.len() < (types.instance_count * options.enum_threshold as usize) / 100;

    // If types are only boolean, skip enum
    let only_bool = types
        .values
        .iter()
        .all(|v| matches!(v, Literals::Boolean(_)));

    // If unique values are below given maximum
    let below_maximum = match options.enum_maximum {
        Some(m) => types.values.len() < m.into(),
        None => true,
    };

    let create_enum = options.use_enum && unique_threshold && !only_bool && below_maximum;

    let create_const = options.use_const && (types.values.len() == 1);

    if create_const {
        json!({"const": literal_to_value(types.values.into_iter().next().unwrap())})
    } else if create_enum {
        json!({"types": type_part, "enum": types.values.into_iter().map(literal_to_value).collect::<Vec<_>>()})
    } else {
        json!({"type": type_part})
    }
}

fn subschema_to_json(
    Subschema {
        types,
        array,
        object,
    }: Subschema,
    options: &CodegenOptions,
) -> Value {
    let mut schemas: Vec<Value> = Vec::new();

    // Terminal cases

    if let Some(t) = types {
        schemas.push(literals_to_json(t, options));
    };

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

    if schemas.is_empty() {
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
            .unwrap_or_else(|_| panic!("Values unable to be printed {:?}", values))
    }
}
