use crate::state::Subschema;

mod jsonschema;

pub use jsonschema::JsonSchema;

pub struct CodegenOptions {
    pub title: Option<String>,
    pub use_enum: bool,
    pub use_const: bool,
    pub enum_threshold: u8,
    pub enum_maximum: Option<u8>,
}

impl CodegenOptions {
    pub fn new() -> Self {
        Self {
            title: None,
            use_enum: true,
            use_const: true,
            enum_threshold: 1,
            enum_maximum: None,
        }
    }
}

pub trait Generation {
    fn generate(object: Subschema, options: CodegenOptions) -> String;
}
