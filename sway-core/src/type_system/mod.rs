mod collect_types_metadata;
mod copy_types;
mod create_type_id;
mod length;
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
mod unconstrained_type_parameters;
mod unify;

pub(crate) use collect_types_metadata::*;
pub(crate) use copy_types::*;
pub(crate) use create_type_id::*;
pub use length::*;
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
pub(crate) use unconstrained_type_parameters::*;

use crate::error::*;
use std::fmt::Debug;

#[cfg(test)]
use sway_types::{integer_bits::IntegerBits, Span};

#[test]
fn generic_enum_resolution() {
    use crate::{language::ty, span::Span, transform, Ident};
    let engine = TypeEngine::default();

    let sp = Span::dummy();

    let generic_type = engine.insert_type_always(TypeInfo::UnknownGeneric {
        name: Ident::new_with_override("T", sp.clone()),
        trait_constraints: VecSet(Vec::new()),
    });
    let variant_types = vec![ty::TyEnumVariant {
        name: Ident::new_with_override("a", sp.clone()),
        tag: 0,
        type_id: generic_type,
        initial_type_id: generic_type,
        span: sp.clone(),
        type_span: sp.clone(),
        attributes: transform::AttributesMap::default(),
    }];
    let ty_1 = engine.insert_type_always(TypeInfo::Enum {
        name: Ident::new_with_override("Result", sp.clone()),
        variant_types,
        type_parameters: vec![TypeParameter {
            type_id: generic_type,
            initial_type_id: generic_type,
            name_ident: Ident::new_no_span("T"),
            trait_constraints: vec![],
            trait_constraints_span: Span::dummy(),
        }],
    });

    let boolean_type = engine.insert_type_always(TypeInfo::Boolean);
    let variant_types = vec![ty::TyEnumVariant {
        name: Ident::new_with_override("a", sp.clone()),
        tag: 0,
        type_id: boolean_type,
        initial_type_id: boolean_type,
        span: sp.clone(),
        type_span: sp.clone(),
        attributes: transform::AttributesMap::default(),
    }];
    let ty_2 = engine.insert_type_always(TypeInfo::Enum {
        name: Ident::new_with_override("Result", sp.clone()),
        variant_types,
        type_parameters: vec![TypeParameter {
            type_id: boolean_type,
            initial_type_id: boolean_type,
            name_ident: Ident::new_no_span("T"),
            trait_constraints: vec![],
            trait_constraints_span: Span::dummy(),
        }],
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
        assert!(matches!(
            engine.look_up_type_id(variant_types[0].type_id),
            TypeInfo::Boolean
        ));
    } else {
        panic!()
    }
}

#[test]
fn basic_numeric_unknown() {
    let engine = TypeEngine::default();

    let sp = Span::dummy();
    // numerics
    let id = engine.insert_type_always(TypeInfo::Numeric);
    let id2 = engine.insert_type_always(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engine.unify(id, id2, &sp, "");
    assert!(errors.is_empty());

    assert!(matches!(
        engine.to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}

#[test]
fn unify_numerics() {
    let engine = TypeEngine::default();
    let sp = Span::dummy();

    // numerics
    let id = engine.insert_type_always(TypeInfo::Numeric);
    let id2 = engine.insert_type_always(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engine.unify(id2, id, &sp, "");
    assert!(errors.is_empty());

    assert!(matches!(
        engine.to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}

#[test]
fn unify_numerics_2() {
    let engine = TypeEngine::default();
    let sp = Span::dummy();

    // numerics
    let id = engine.insert_type_always(TypeInfo::Numeric);
    let id2 = engine.insert_type_always(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engine.unify(id, id2, &sp, "");
    assert!(errors.is_empty());

    assert!(matches!(
        engine.to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}
