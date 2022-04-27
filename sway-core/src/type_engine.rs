use crate::error::*;
use std::iter::FromIterator;
use sway_types::span::Span;

mod engine;
mod integer_bits;
mod type_info;
mod unresolved_type_check;
pub use engine::*;
pub use integer_bits::*;
use sway_types::Property;
pub use type_info::*;
pub(crate) use unresolved_type_check::UnresolvedTypeCheck;

/// A identifier to uniquely refer to our type terms
pub type TypeId = usize;

pub(crate) trait JsonAbiString {
    fn json_abi_str(&self) -> String;
}

impl JsonAbiString for TypeId {
    fn json_abi_str(&self) -> String {
        look_up_type_id(*self).json_abi_str()
    }
}

pub(crate) trait FriendlyTypeString {
    fn friendly_type_str(&self) -> String;
}

impl FriendlyTypeString for TypeId {
    fn friendly_type_str(&self) -> String {
        look_up_type_id(*self).friendly_type_str()
    }
}

pub(crate) trait ToJsonAbi {
    fn generate_json_abi(&self) -> Option<Vec<Property>>;
}

impl ToJsonAbi for TypeId {
    fn generate_json_abi(&self) -> Option<Vec<Property>> {
        match look_up_type_id(*self) {
            TypeInfo::Struct { fields, .. } => {
                Some(fields.iter().map(|x| x.generate_json_abi()).collect())
            }
            TypeInfo::Enum { variant_types, .. } => Some(
                variant_types
                    .iter()
                    .map(|x| x.generate_json_abi())
                    .collect(),
            ),
            _ => None,
        }
    }
}

#[test]
fn generic_enum_resolution() {
    use crate::semantic_analysis::ast_node::TypedEnumVariant;
    use crate::Ident;
    let engine = Engine::default();

    let sp = Span::dummy();

    let variant_types = vec![TypedEnumVariant {
        name: Ident::new_with_override("a", sp.clone()),
        tag: 0,
        r#type: engine.insert_type(TypeInfo::UnknownGeneric {
            name: Ident::new_with_override("T", sp.clone()),
        }),
        span: sp.clone(),
    }];

    let ty_1 = engine.insert_type(TypeInfo::Enum {
        name: Ident::new_with_override("Result", sp.clone()),
        variant_types,
    });

    let variant_types = vec![TypedEnumVariant {
        name: Ident::new_with_override("a", sp.clone()),
        tag: 0,
        r#type: engine.insert_type(TypeInfo::Boolean),
        span: sp.clone(),
    }];

    let ty_2 = engine.insert_type(TypeInfo::Enum {
        name: Ident::new_with_override("Result", sp.clone()),
        variant_types,
    });

    // Unify them together...
    let (_, errors) = engine.unify(ty_1, ty_2, &sp, "");
    assert!(errors.is_empty());

    if let TypeInfo::Enum {
        name,
        variant_types,
    } = engine.look_up_type_id(ty_1)
    {
        assert_eq!(name.as_str(), "Result");
        assert_eq!(
            engine.look_up_type_id(variant_types[0].r#type),
            TypeInfo::Boolean
        );
    } else {
        panic!()
    }
}

#[test]
fn basic_numeric_unknown() {
    let engine = Engine::default();

    let sp = Span::dummy();
    // numerics
    let id = engine.insert_type(TypeInfo::Numeric);
    let id2 = engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engine.unify(id, id2, &sp, "");
    assert!(errors.is_empty());

    assert_eq!(
        engine.resolve_type(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    );
}
#[test]
fn chain_of_refs() {
    let engine = Engine::default();
    let sp = Span::dummy();
    // numerics
    let id = engine.insert_type(TypeInfo::Numeric);
    let id2 = engine.insert_type(TypeInfo::Ref(id));
    let id3 = engine.insert_type(TypeInfo::Ref(id));
    let id4 = engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engine.unify(id4, id2, &sp, "");
    assert!(errors.is_empty());

    assert_eq!(
        engine.resolve_type(id3, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    );
}
#[test]
fn chain_of_refs_2() {
    let engine = Engine::default();
    let sp = Span::dummy();
    // numerics
    let id = engine.insert_type(TypeInfo::Numeric);
    let id2 = engine.insert_type(TypeInfo::Ref(id));
    let id3 = engine.insert_type(TypeInfo::Ref(id));
    let id4 = engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engine.unify(id2, id4, &sp, "");
    assert!(errors.is_empty());

    assert_eq!(
        engine.resolve_type(id3, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    );
}

fn parse_str_type(raw: &str, span: Span) -> CompileResult<TypeInfo> {
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
    match parse_str_type("str[20]", Span::dummy()).value {
        Some(value) if value == TypeInfo::Str(20) => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("str[]", Span::dummy()).value {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("str[ab]", Span::dummy()).value {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("str [ab]", Span::dummy()).value {
        None => (),
        _ => panic!("failed test"),
    }

    match parse_str_type("not even a str[ type", Span::dummy()).value {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("", Span::dummy()).value {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("20", Span::dummy()).value {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("[20]", Span::dummy()).value {
        None => (),
        _ => panic!("failed test"),
    }
}
