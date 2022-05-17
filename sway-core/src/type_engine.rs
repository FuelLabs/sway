use crate::error::*;
use std::fmt::{Debug, Display};

mod engine;
mod integer_bits;
mod type_info;
mod unresolved_type_check;
pub use engine::*;
use fuels_types::Property;
pub use integer_bits::*;
pub use type_info::*;
pub(crate) use unresolved_type_check::UnresolvedTypeCheck;

/// A identifier to uniquely refer to our type terms
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct TypeId(usize);

impl std::ops::Deref for TypeId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&look_up_type_id(*self).friendly_type_str())
    }
}

impl Debug for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&look_up_type_id(*self).friendly_type_str())
    }
}

impl From<usize> for TypeId {
    fn from(o: usize) -> Self {
        TypeId(o)
    }
}
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
    use crate::{span::Span, Ident};
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
        ..
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
    use sway_types::Span;
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
    use sway_types::Span;
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
    use sway_types::Span;
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
