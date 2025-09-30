use super::Generation;
use crate::state::{FieldState, StateObject, UnionObject};

const IMPORTS: &'static str = "from dataclasses import dataclass\n\n";
const CLASS_HEADER: &'static str = "@dataclass\nclass ";
const INDENT: &'static str = "  ";
const COLON: &'static str = ":";
const SPACE: &'static str = " ";
const NL: &'static str = "\n";
const ARRAY: &'static str = "list";
const LBRACKET: &'static str = "[";
const RBRACKET: &'static str = "]";

pub struct Python {}

impl Generation for Python {
    fn generate(object: StateObject) -> String {
        let uo = UnionObject::from_state_object(object);
        let mut output = String::new();
        output = output + IMPORTS;
        codegen_union(&mut output, "Entry", uo);

        output
    }
}

fn python_types(state: FieldState) -> &'static str {
    match state {
        FieldState::Unset => "None",
        FieldState::None => "None",
        FieldState::Bool => "bool",
        FieldState::Int => "int",
        FieldState::Float => "float",
        FieldState::Str => "str",
        FieldState::BoolOrNone => "bool | None",
        FieldState::IntOrNone => "int | None",
        FieldState::FloatOrNone => "float | None",
        FieldState::StrOrNone => "str | None",
    }
}

fn codegen_union(output_text: &mut String, field_name: &str, uo: UnionObject) -> String {
    let mut output = Vec::new();

    if let Some(terminal) = uo.terminal {
        output.push(String::new() + python_types(terminal));
    }

    if let Some(array) = uo.array {
        output.push(
            String::new()
                + ARRAY
                + LBRACKET
                + &codegen_union(output_text, field_name, *array)
                + RBRACKET,
        );
    }

    if let Some(object) = uo.object {
        let mut nested_output = String::new();

        nested_output = nested_output + CLASS_HEADER + field_name + COLON + NL;

        for (k, v) in object {
            nested_output = nested_output
                + INDENT
                + &k
                + COLON
                + SPACE
                + &codegen_union(output_text, k.as_str(), v)
                + NL;
        }

        nested_output += NL;

        output_text.push_str(nested_output.as_str());
        output.push(field_name.to_owned());
    }

    output.join(" | ")
}
