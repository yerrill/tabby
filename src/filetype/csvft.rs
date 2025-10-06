use std::error::Error;

use super::{Filetype, StateObject};
use crate::state;

pub struct CsvOptions {
    pub delimiter: char,
}

impl CsvOptions {
    pub fn new() -> Self {
        Self { delimiter: ',' }
    }
}

pub struct CsvFileType {
    fields: Vec<(String, state::FieldState)>,
}

impl CsvFileType {
    pub fn new(file: &str, options: CsvOptions) -> Result<Self, Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(options.delimiter.try_into()?)
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

    #[allow(dead_code)]
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
    use crate::state::FieldState;

    #[test]
    fn change1() {
        let table = [
            ["h1", "h2", "h3"],
            ["a", "1", "true"],
            ["b", "2", "false"],
            ["", "1.1", ""],
        ];

        let mut fields = vec![
            (String::from("h1"), FieldState::Unset),
            (String::from("h2"), FieldState::Unset),
            (String::from("h3"), FieldState::Unset),
        ];

        assert!(fields[0] == ("h1".to_owned(), state::FieldState::Unset));
        assert!(fields[1] == ("h2".to_owned(), state::FieldState::Unset));
        assert!(fields[2] == ("h3".to_owned(), state::FieldState::Unset));

        fields = CsvFileType::update_states(fields, table[1].into_iter());

        assert!(fields[0] == ("h1".to_owned(), state::FieldState::Str));
        assert!(fields[1] == ("h2".to_owned(), state::FieldState::Int));
        assert!(fields[2] == ("h3".to_owned(), state::FieldState::Bool));

        fields = CsvFileType::update_states(fields, table[2].into_iter());

        assert!(fields[0] == ("h1".to_owned(), state::FieldState::Str));
        assert!(fields[1] == ("h2".to_owned(), state::FieldState::Int));
        assert!(fields[2] == ("h3".to_owned(), state::FieldState::Bool));

        fields = CsvFileType::update_states(fields, table[3].into_iter());

        assert!(fields[0] == ("h1".to_owned(), state::FieldState::StrOrNone));
        assert!(fields[1] == ("h2".to_owned(), state::FieldState::Float));
        assert!(fields[2] == ("h3".to_owned(), state::FieldState::BoolOrNone));

        assert!(fields.len() == 3);
    }
}
