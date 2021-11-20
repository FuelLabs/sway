use crate::{error::*, Span};

use std::iter::FromIterator;

mod engine;
mod integer_bits;
mod type_info;
pub(crate) use engine::*;
pub use integer_bits::*;
pub use type_info::*;

pub trait TypeEngine<'sc> {
    type TypeId;
    type TypeInfo;
    type ResolvedType;
    type Error;
    /// Insert a new bit of inference information about a specific type. Receive a [Self::TypeId]
    /// representing that information in return.
    fn insert(&mut self, info: Self::TypeInfo) -> Self::TypeId;
    /// Attempt to unify two type ids into one equivalence class. Throw an error if it is impossible.
    fn unify(
        &mut self,
        a: Self::TypeId,
        b: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Option<Warning<'sc>>, Self::Error>;
    /// Like `unify`, but also takes a self type in case either type is Self.
    fn unify_with_self(
        &mut self,
        a: Self::TypeId,
        b: Self::TypeId,
        self_type: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Option<Warning<'sc>>, Self::Error>;
    /// Attempt to reconstruct a concrete type from the given type term ID. This
    /// may fail if we don't yet have enough information to figure out what the
    /// type is.
    fn resolve(&self, id: Self::TypeId) -> Result<Self::TypeInfo, Self::Error>;

    /// Looks up a type id. Panics if the given ID doesn't exist in the type
    /// engine.
    fn look_up_type_id(&self, id: Self::TypeId) -> TypeInfo;
}

/// A identifier to uniquely refer to our type terms
pub type TypeId = usize;

pub(crate) trait FriendlyTypeString {
    fn friendly_type_str(&self) -> String;
}

impl FriendlyTypeString for TypeId {
    fn friendly_type_str(&self) -> String {
        let res = look_up_type_id(*self).friendly_type_str();
        res
    }
}

#[test]
fn basic_numeric_unknown() {
    let mut engine = Engine::default();

    let sp = Span {
        span: pest::Span::new(" ", 0, 0).unwrap(),
        path: None,
    };
    // numerics
    let id = engine.insert(TypeInfo::Numeric);
    let id2 = engine.insert(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    engine.unify(id, id2, &sp).unwrap();

    assert_eq!(
        engine.resolve(id).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    );
}
#[test]
fn chain_of_refs() {
    let mut engine = Engine::default();
    let sp = Span {
        span: pest::Span::new(" ", 0, 0).unwrap(),
        path: None,
    };
    // numerics
    let id = engine.insert(TypeInfo::Numeric);
    let id2 = engine.insert(TypeInfo::Ref(id));
    let id3 = engine.insert(TypeInfo::Ref(id));
    let id4 = engine.insert(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    engine.unify(id4, id2, &sp).unwrap();

    assert_eq!(
        engine.resolve(id3).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    );
}
#[test]
fn chain_of_refs_2() {
    let mut engine = Engine::default();
    let sp = Span {
        span: pest::Span::new(" ", 0, 0).unwrap(),
        path: None,
    };
    // numerics
    let id = engine.insert(TypeInfo::Numeric);
    let id2 = engine.insert(TypeInfo::Ref(id));
    let id3 = engine.insert(TypeInfo::Ref(id));
    let id4 = engine.insert(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    engine.unify(id2, id4, &sp).unwrap();

    assert_eq!(
        engine.resolve(id3).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    );
}

fn parse_str_type<'sc>(raw: &'sc str, span: Span<'sc>) -> CompileResult<'sc, TypeInfo> {
    if raw.starts_with("str[") {
        let mut rest = raw.split_at("str[".len()).1.chars().collect::<Vec<_>>();
        if let Some(']') = rest.pop() {
            if let Ok(num) = String::from_iter(rest).parse() {
                return ok(TypeInfo::Str(num), vec![], vec![]);
            }
        }
        return err(
            vec![],
            vec![CompileError::InvalidStrType {
                raw: raw.to_string(),
                span,
            }],
        );
    }
    err(vec![], vec![CompileError::UnknownType { span }])
}

#[test]
fn test_str_parse() {
    match parse_str_type(
        "str[20]",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        Some(value) if value == TypeInfo::Str(20) => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "str[]",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "str[ab]",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "str [ab]",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }

    match parse_str_type(
        "not even a str[ type",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "20",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "[20]",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
}
