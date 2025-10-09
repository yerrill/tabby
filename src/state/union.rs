use super::{FieldState, StateObject};
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq, Debug)]
pub struct UnionObject {
    pub terminal: HashSet<FieldState>,
    pub array: Option<Box<UnionObject>>,
    pub object: Option<HashMap<String, UnionObject>>,
}

impl UnionObject {
    pub fn from_state_object(st: StateObject) -> UnionObject {
        match st {
            StateObject::Type(t) => UnionObject {
                terminal: Some(FieldState::Unset.change(t)),
                array: None,
                object: None,
            },
            StateObject::Array(a) => UnionObject {
                terminal: None,
                array: Some(Box::new(
                    a.into_iter()
                        .map(|i| Self::from_state_object(i))
                        .reduce(|acc, e| crunch_unions(acc, e))
                        .unwrap_or(UnionObject {
                            terminal: None,
                            array: None,
                            object: None,
                        }),
                )),
                object: None,
            },
            StateObject::Object(o) => UnionObject {
                terminal: None,
                array: None,
                object: Some(
                    o.into_iter()
                        .map(|(k, v)| (k, Self::from_state_object(v)))
                        .collect(),
                ),
            },
        }
    }
}

fn crunch_unions(uo_1: UnionObject, uo_2: UnionObject) -> UnionObject {
    let terminal = match (uo_1.terminal, uo_2.terminal) {
        (Some(s1), Some(s2)) => Some(s1.change(s2)),
        (Some(s1), None) => Some(s1),
        (None, Some(s2)) => Some(s2),
        (None, None) => None,
    };

    let array = match (uo_1.array, uo_2.array) {
        (Some(s1), Some(s2)) => Some(Box::new(crunch_unions(*s1, *s2))),
        (Some(s1), None) => Some(s1),
        (None, Some(s2)) => Some(s2),
        (None, None) => None,
    };

    let object = match (uo_1.object, uo_2.object) {
        (Some(s1), Some(mut s2)) => {
            let mut new_map = HashMap::new();
            //Does it matter if the a unionobject is in 1 but not the other map
            //Combine with None terminal?

            for (key, s1_value) in s1 {
                let Some(s2_value) = s2.remove(&key) else {
                    new_map.insert(
                        key,
                        crunch_unions(
                            s1_value,
                            UnionObject {
                                terminal: Some(FieldState::None),
                                array: None,
                                object: None,
                            },
                        ),
                    );
                    continue;
                };

                new_map.insert(key, crunch_unions(s1_value, s2_value));
            }

            for (key, value) in s2 {
                new_map.insert(
                    key,
                    crunch_unions(
                        value,
                        UnionObject {
                            terminal: Some(FieldState::None),
                            array: None,
                            object: None,
                        },
                    ),
                );
            }

            Some(new_map)
        }
        (Some(s1), None) => Some(s1),
        (None, Some(s2)) => Some(s2),
        (None, None) => None,
    };

    UnionObject {
        terminal,
        array,
        object,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::FieldState;

    #[test]
    fn union_obj() {
        let u1 = UnionObject {
            terminal: Some(FieldState::Int),
            array: Some(Box::new(UnionObject {
                terminal: Some(FieldState::Bool),
                array: None,
                object: None,
            })),
            object: Some(
                vec![
                    (
                        String::from("a"),
                        UnionObject {
                            terminal: Some(FieldState::Float),
                            array: None,
                            object: None,
                        },
                    ),
                    (
                        String::from("b"),
                        UnionObject {
                            terminal: Some(FieldState::Int),
                            array: None,
                            object: None,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
            ),
        };

        let u2 = UnionObject {
            terminal: Some(FieldState::None),
            array: Some(Box::new(UnionObject {
                terminal: Some(FieldState::Bool),
                array: None,
                object: None,
            })),
            object: Some(
                vec![
                    (
                        String::from("a"),
                        UnionObject {
                            terminal: Some(FieldState::Float),
                            array: None,
                            object: None,
                        },
                    ),
                    (
                        String::from("c"),
                        UnionObject {
                            terminal: Some(FieldState::Int),
                            array: None,
                            object: None,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
            ),
        };

        let out = crunch_unions(u1, u2);

        assert_eq!(out.terminal, Some(FieldState::IntOrNone));
        assert_eq!(
            out.array,
            Some(Box::new(UnionObject {
                terminal: Some(FieldState::Bool),
                array: None,
                object: None,
            }))
        );
        assert_eq!(
            out.object,
            Some(
                vec![
                    (
                        String::from("a"),
                        UnionObject {
                            terminal: Some(FieldState::Float),
                            array: None,
                            object: None,
                        },
                    ),
                    (
                        String::from("b"),
                        UnionObject {
                            terminal: Some(FieldState::IntOrNone),
                            array: None,
                            object: None,
                        },
                    ),
                    (
                        String::from("c"),
                        UnionObject {
                            terminal: Some(FieldState::IntOrNone),
                            array: None,
                            object: None,
                        },
                    ),
                ]
                .into_iter()
                .collect()
            ),
        );
    }

    #[test]
    fn condense() {
        let a = StateObject::Object(
            vec![
                (String::from("a"), StateObject::Type(FieldState::Int)),
                (
                    String::from("b"),
                    StateObject::Array(vec![
                        StateObject::Type(FieldState::Float),
                        StateObject::Type(FieldState::Int),
                    ]),
                ),
            ]
            .into_iter()
            .collect(),
        );

        let a_out = UnionObject {
            terminal: None,
            array: None,
            object: Some(
                vec![
                    (
                        String::from("a"),
                        UnionObject {
                            terminal: Some(FieldState::Int),
                            array: None,
                            object: None,
                        },
                    ),
                    (
                        String::from("b"),
                        UnionObject {
                            terminal: None,
                            array: Some(Box::new(UnionObject {
                                terminal: Some(FieldState::Float),
                                array: None,
                                object: None,
                            })),
                            object: None,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
            ),
        };

        assert_eq!(UnionObject::from_state_object(a), a_out);
    }
}
