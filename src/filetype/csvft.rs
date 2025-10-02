use std::error::Error;

use super::{Filetype, StateObject};
use crate::state;

pub struct CsvOptions {
    pub delimiter: u8,
}

impl CsvOptions {
    pub fn new() -> Self {
        Self { delimiter: b',' }
    }
}

pub struct CsvFileType {
    fields: Vec<(String, state::FieldState)>,
}

impl CsvFileType {
    pub fn new(file: &str, options: CsvOptions) -> Result<Self, Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(options.delimiter)
            .from_reader(file.as_bytes());

        let mut fields = reader
            .headers()?
            .iter()
            .map(|s| (s.to_owned(), state::FieldState::Unset))
            .collect();

        for result in reader.records() {
            fields = Self::update_states(fields, result?.into_iter());
        }

        Ok(Self { fields })
    }

    fn update_states<'a>(
        fields: Vec<(String, state::FieldState)>,
        mut i: impl Iterator<Item = &'a str>,
    ) -> Vec<(String, state::FieldState)> {
        let updated_fields = fields
            .into_iter()
            .map(|(header, state)| {
                let new_value = i.next();

                match new_value {
                    Some(v) => (header, state.change(state::FieldState::from_str(v))),
                    None => (header, state.change(state::FieldState::None)),
                }
            })
            .collect();

        updated_fields
    }

    pub fn to_string(&self) -> String {
        let cols = self
            .fields
            .iter()
            .map(|(h, s)| format!("{} ({})", h, s.to_str()))
            .collect::<Vec<_>>()
            .join(", ");

        cols
    }
}

impl Filetype for CsvFileType {
    fn to_object(self) -> StateObject {
        let entries = StateObject::Object(
            self.fields
                .into_iter()
                .map(|(header, state)| (header, StateObject::Type(state)))
                .collect(),
        );

        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state;

    #[test]
    fn change1() {
        let table = [
            ["h1", "h2", "h3"],
            ["a", "1", "true"],
            ["b", "2", "false"],
            ["", "1.1", ""],
        ];

        let mut fields =
            CsvFileType::new(table[0].iter().map(|&s| s.to_owned()).collect()).unwrap();

        assert!(fields.fields[0] == ("h1".to_owned(), state::FieldState::Unset));
        assert!(fields.fields[1] == ("h2".to_owned(), state::FieldState::Unset));
        assert!(fields.fields[2] == ("h3".to_owned(), state::FieldState::Unset));

        fields = fields.update_states(table[1].into_iter());

        assert!(fields.fields[0] == ("h1".to_owned(), state::FieldState::Str));
        assert!(fields.fields[1] == ("h2".to_owned(), state::FieldState::Int));
        assert!(fields.fields[2] == ("h3".to_owned(), state::FieldState::Bool));

        fields = fields.update_states(table[2].into_iter());

        assert!(fields.fields[0] == ("h1".to_owned(), state::FieldState::Str));
        assert!(fields.fields[1] == ("h2".to_owned(), state::FieldState::Int));
        assert!(fields.fields[2] == ("h3".to_owned(), state::FieldState::Bool));

        fields = fields.update_states(table[3].into_iter());

        assert!(fields.fields[0] == ("h1".to_owned(), state::FieldState::StrOrNone));
        assert!(fields.fields[1] == ("h2".to_owned(), state::FieldState::Float));
        assert!(fields.fields[2] == ("h3".to_owned(), state::FieldState::BoolOrNone));

        assert!(fields.fields.len() == 3);
    }
}
