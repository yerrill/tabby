use crate::state::StateObject;

mod jsonschema;
mod python;

pub use python::Python;

pub trait Generation {
    fn generate(object: StateObject) -> String;
}
