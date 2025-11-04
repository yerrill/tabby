use super::{DataValues, Literals};
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq, Debug)]
pub struct ObjectProperty {
    pub value: Subschema,
    pub required: bool,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Subschema {
    pub types: HashSet<Literals>,
    pub array: Option<Box<Subschema>>,
    pub object: Option<HashMap<String, ObjectProperty>>,
    pub types_instance_count: usize,
}

impl Subschema {
    pub fn new() -> Self {
        Self {
            types: HashSet::new(),
            array: None,
            object: None,
            types_instance_count: 0,
        }
    }

    pub fn from_data(data: DataValues) -> Self {
        match data {
            DataValues::Literal(t) => Self {
                types: HashSet::from([t]),
                array: None,
                object: None,
                types_instance_count: 1,
            },
            DataValues::Array(a) => Self {
                types: HashSet::new(),
                array: Some(Box::new(
                    a.into_iter()
                        .map(Self::from_data)
                        .reduce(crunch_schemas)
                        .unwrap_or(Self::new()),
                )),
                object: None,
                types_instance_count: 0,
            },
            DataValues::Object(o) => Self {
                types: HashSet::new(),
                array: None,
                object: Some(
                    o.into_iter()
                        .map(|(k, v)| {
                            (
                                k,
                                ObjectProperty {
                                    value: Self::from_data(v),
                                    required: true,
                                },
                            )
                        })
                        .collect(),
                ),
                types_instance_count: 0,
            },
        }
    }
}

fn crunch_schemas(uo_1: Subschema, uo_2: Subschema) -> Subschema {
    let types = {
        let mut set = HashSet::new();
        set.extend(uo_1.types);
        set.extend(uo_2.types);
        set
    };

    let array = match (uo_1.array, uo_2.array) {
        (Some(s1), Some(s2)) => Some(Box::new(crunch_schemas(*s1, *s2))),
        (Some(s1), None) => Some(s1),
        (None, Some(s2)) => Some(s2),
        (None, None) => None,
    };

    let object = match (uo_1.object, uo_2.object) {
        (Some(s1), Some(mut s2)) => {
            let mut new_map: HashMap<String, ObjectProperty> = HashMap::new();

            for (
                key,
                ObjectProperty {
                    value: sub_1,
                    required: req_1,
                },
            ) in s1
            {
                let Some(ObjectProperty {
                    value: sub_2,
                    required: req_2,
                }) = s2.remove(&key)
                else {
                    // If key not in other object, insert self with required false
                    new_map.insert(
                        key,
                        ObjectProperty {
                            value: sub_1,
                            required: false,
                        },
                    );
                    continue;
                };

                // If both keys exist, crunch schemas and AND requirement flag
                let new_sub = crunch_schemas(sub_1, sub_2);

                new_map.insert(
                    key,
                    ObjectProperty {
                        value: new_sub,
                        required: req_1 && req_2,
                    },
                );
            }

            // Process any remaining keys in s2
            for (
                key,
                ObjectProperty {
                    value: sub,
                    required: _,
                },
            ) in s2
            {
                new_map.insert(
                    key,
                    ObjectProperty {
                        value: sub,
                        required: false,
                    },
                );
            }

            Some(new_map)
        }
        (Some(s1), None) => Some(s1),
        (None, Some(s2)) => Some(s2),
        (None, None) => None,
    };

    Subschema {
        types,
        array,
        object,
        types_instance_count: uo_1.types_instance_count + uo_2.types_instance_count,
    }
}

#[cfg(test)]
#[allow(dead_code, unused_imports)]
mod tests {
    use std::collections::HashSet;

    use super::super::{DataValues, Literals};
    use super::{ObjectProperty, Subschema, crunch_schemas};

    fn nul() -> DataValues {
        DataValues::Literal(Literals::Null)
    }

    fn bol(b: bool) -> DataValues {
        DataValues::Literal(Literals::Boolean(b))
    }

    fn int(a: i64) -> DataValues {
        DataValues::Literal(Literals::Integer(a))
    }

    fn flt(f: f64) -> DataValues {
        DataValues::Literal(Literals::Float(f.to_bits()))
    }

    fn arr(items: &[DataValues]) -> DataValues {
        DataValues::Array(Vec::from(items))
    }

    fn obj(pairs: &[(&str, DataValues)]) -> DataValues {
        DataValues::Object(
            Vec::from(pairs)
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect(),
        )
    }

    #[test]
    fn convert() {
        assert_eq!(
            Subschema::from_data(nul()),
            Subschema {
                types: HashSet::from([Literals::Null]),
                types_instance_count: 1,
                ..Subschema::new()
            }
        );

        assert_eq!(
            Subschema::from_data(arr(&[bol(true), bol(false)])),
            Subschema {
                array: Some(Box::new(Subschema {
                    types: HashSet::from([Literals::Boolean(true), Literals::Boolean(false)]),
                    types_instance_count: 2,
                    ..Subschema::new()
                })),
                ..Subschema::new()
            }
        );
    }

    #[test]
    fn crunch() {}
}
