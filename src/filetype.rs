use crate::state::StateObject;

pub mod csvft;
pub mod jsonft;

pub use csvft::CSVFileType;
pub use jsonft::JsonFileType;

pub trait Filetype {
    fn to_object(self) -> StateObject;
}
