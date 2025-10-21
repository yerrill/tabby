use crate::state::DataValues;

mod csvft;
mod jsonft;

pub use csvft::{CsvFileType, CsvOptions};
pub use jsonft::JsonFileType;

pub trait Filetype {
    fn to_object(self) -> DataValues;
}
