use crate::state::StateObject;

mod jsonschema;
mod python;

pub use jsonschema::JsonSchema;
pub use python::Python;

pub struct CodegenOptions {
    pub title: Option<String>,
}

impl CodegenOptions {
    pub fn new() -> Self {
        Self { title: None }
    }
}

pub trait Generation {
    fn generate(object: StateObject, options: CodegenOptions) -> String;
}
