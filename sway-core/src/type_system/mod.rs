mod collect_types_metadata;
mod copy_types;
mod create_type_id;
mod integer_bits;
mod replace_self_type;
mod resolved_type;
mod trait_constraint;
mod type_argument;
mod type_binding;
mod type_engine;
mod type_id;
mod type_info;
mod type_mapping;
mod type_parameter;

pub(crate) use collect_types_metadata::*;
pub(crate) use copy_types::*;
pub(crate) use create_type_id::*;
pub use integer_bits::*;
pub(crate) use replace_self_type::*;
pub(crate) use resolved_type::*;
pub(crate) use trait_constraint::*;
pub use type_argument::*;
pub(crate) use type_binding::*;
pub use type_engine::*;
pub use type_id::*;
pub use type_info::*;
pub(crate) use type_mapping::*;
pub use type_parameter::*;

use crate::error::*;
use std::fmt::Debug;

#[test]
fn generic_enum_resolution() {
    use crate::semantic_analysis::ast_node::TypedEnumVariant;
    use crate::{span::Span, Ident};
    let engine = TypeEngine::default();

    let sp = Span::dummy();

    let variant_types = vec![TypedEnumVariant {
        name: Ident::new_with_override("a", sp.clone()),
        tag: 0,
        type_id: engine.insert_type(TypeInfo::UnknownGeneric {
            name: Ident::new_with_override("T", sp.clone()),
        }),
        initial_type_id: engine.insert_type(TypeInfo::UnknownGeneric {
            name: Ident::new_with_override("T", sp.clone()),
        }),
        span: sp.clone(),
        type_span: sp.clone(),
    }];

    let ty_1 = engine.insert_type(TypeInfo::Enum {
        name: Ident::new_with_override("Result", sp.clone()),
        variant_types,
        type_parameters: vec![],
    });

    let variant_types = vec![TypedEnumVariant {
        name: Ident::new_with_override("a", sp.clone()),
        tag: 0,
        type_id: engine.insert_type(TypeInfo::Boolean),
        initial_type_id: engine.insert_type(TypeInfo::Boolean),
        span: sp.clone(),
        type_span: sp.clone(),
    }];

    let ty_2 = engine.insert_type(TypeInfo::Enum {
        name: Ident::new_with_override("Result", sp.clone()),
        variant_types,
        type_parameters: vec![],
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
            engine.look_up_type_id(variant_types[0].type_id),
            TypeInfo::Boolean
        );
    } else {
        panic!()
    }
}

#[test]
fn basic_numeric_unknown() {
    use sway_types::Span;
    let engine = TypeEngine::default();

    let sp = Span::dummy();
    // numerics
    let id = engine.insert_type(TypeInfo::Numeric);
    let id2 = engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engine.unify(id, id2, &sp, "");
    assert!(errors.is_empty());

    assert_eq!(
        engine
            .to_typeinfo(id, &Span::dummy())
            .unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    );
}
#[test]
fn chain_of_refs() {
    use sway_types::Span;
    let engine = TypeEngine::default();
    let sp = Span::dummy();
    // numerics
    let id = engine.insert_type(TypeInfo::Numeric);
    let id2 = engine.insert_type(TypeInfo::Ref(id, sp.clone()));
    let id3 = engine.insert_type(TypeInfo::Ref(id, sp.clone()));
    let id4 = engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engine.unify(id4, id2, &sp, "");
    assert!(errors.is_empty());

    assert_eq!(
        engine
            .to_typeinfo(id3, &Span::dummy())
            .unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    );
}
#[test]
fn chain_of_refs_2() {
    use sway_types::Span;
    let engine = TypeEngine::default();
    let sp = Span::dummy();
    // numerics
    let id = engine.insert_type(TypeInfo::Numeric);
    let id2 = engine.insert_type(TypeInfo::Ref(id, sp.clone()));
    let id3 = engine.insert_type(TypeInfo::Ref(id, sp.clone()));
    let id4 = engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engine.unify(id2, id4, &sp, "");
    assert!(errors.is_empty());

    assert_eq!(
        engine
            .to_typeinfo(id3, &Span::dummy())
            .unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    );
}
