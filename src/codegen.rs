use crate::state::Subschema;

mod jsonschema;

pub use jsonschema::JsonSchema;

pub struct CodegenOptions {
    pub title: Option<String>,
    pub enum_threshold: u8,
}

impl CodegenOptions {
    pub fn new() -> Self {
        Self {
            title: None,
            enum_threshold: 40,
        }
    }
}

pub trait Generation {
    fn generate(object: Subschema, options: CodegenOptions) -> String;
}
