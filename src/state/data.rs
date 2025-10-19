use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Literals {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(u64),
    String(String),
}

#[derive(Clone, Debug)]
pub enum DataValues {
    Literal(Literals),
    Array(Vec<DataValues>),
    Object(HashMap<String, DataValues>),
}

#[cfg(test)]
mod tests {
    #[test]
    fn convert_f64() {
        assert_eq!(f64::from_bits((1f64).to_bits()), 1f64);
    }
}
