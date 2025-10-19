use std::{collections::HashMap, error::Error};

use super::Filetype;
use crate::state::{DataValues, Literals};

pub struct CsvOptions {
    pub delimiter: char,
}

impl CsvOptions {
    pub fn new() -> Self {
        Self { delimiter: ',' }
    }
}

pub struct CsvFileType {
    objects: Vec<HashMap<String, Literals>>,
}

impl CsvFileType {
    pub fn new(file: &str, options: CsvOptions) -> Result<Self, Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(options.delimiter.try_into()?)
            .from_reader(file.as_bytes());

        let fields = reader
            .headers()?
            .iter()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let mut objs: Vec<HashMap<String, Literals>> = Vec::new();

        for result in reader.records() {
            objs.push(
                fields
                    .iter()
                    .zip(result?.iter())
                    .map(|(k, v)| (k.to_owned(), Literals::from(v)))
                    .collect(),
            );
        }

        Ok(Self { objects: objs })
    }
}

impl Filetype for CsvFileType {
    fn to_object(self) -> DataValues {
        DataValues::Array(
            self.objects
                .into_iter()
                .map(|h| {
                    DataValues::Object(
                        h.into_iter()
                            .map(|(k, v)| (k, DataValues::Literal(v)))
                            .collect(),
                    )
                })
                .collect(),
        )
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::state::FieldState;
// }
