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
mod unify_check;

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
    use crate::{
        declaration_engine::DeclarationEngine, language::ty, span::Span, transform, Ident,
    };
    let type_engine = TypeEngine::default();
    let declaration_engine = DeclarationEngine::default();

    let sp = Span::dummy();
    let generic_name = Ident::new_with_override("T", sp.clone());
    let a_name = Ident::new_with_override("a", sp.clone());
    let result_name = Ident::new_with_override("Result", sp.clone());

    /*
    Result<_> {
        a: _
    }
    */
    let generic_type = type_engine.insert_type(
        &declaration_engine,
        TypeInfo::UnknownGeneric {
            name: generic_name.clone(),
            trait_constraints: VecSet(Vec::new()),
        },
    );
    let placeholder_type = type_engine.insert_type(
        &declaration_engine,
        TypeInfo::Placeholder(TypeParameter {
            type_id: generic_type,
            initial_type_id: generic_type,
            name_ident: generic_name.clone(),
            trait_constraints: vec![],
            trait_constraints_span: sp.clone(),
        }),
    );
    let placeholder_type_param = TypeParameter {
        type_id: placeholder_type,
        initial_type_id: placeholder_type,
        name_ident: generic_name.clone(),
        trait_constraints: vec![],
        trait_constraints_span: sp.clone(),
    };
    let variant_types = vec![ty::TyEnumVariant {
        name: a_name.clone(),
        tag: 0,
        type_id: placeholder_type,
        initial_type_id: placeholder_type,
        span: sp.clone(),
        type_span: sp.clone(),
        attributes: transform::AttributesMap::default(),
    }];
    let ty_1 = type_engine.insert_type(
        &declaration_engine,
        TypeInfo::Enum {
            name: result_name.clone(),
            variant_types,
            type_parameters: vec![placeholder_type_param],
        },
    );

    /*
    Result<bool> {
        a: bool
    }
    */
    let boolean_type = type_engine.insert_type(&declaration_engine, TypeInfo::Boolean);
    let variant_types = vec![ty::TyEnumVariant {
        name: a_name,
        tag: 0,
        type_id: boolean_type,
        initial_type_id: boolean_type,
        span: sp.clone(),
        type_span: sp.clone(),
        attributes: transform::AttributesMap::default(),
    }];
    let type_param = TypeParameter {
        type_id: boolean_type,
        initial_type_id: boolean_type,
        name_ident: generic_name,
        trait_constraints: vec![],
        trait_constraints_span: sp.clone(),
    };
    let ty_2 = type_engine.insert_type(
        &declaration_engine,
        TypeInfo::Enum {
            name: result_name,
            variant_types,
            type_parameters: vec![type_param],
        },
    );

    // Unify them together...
    let (_, errors) = type_engine.unify(&declaration_engine, ty_1, ty_2, &sp, "", None);
    for err in errors.iter() {
        println!("{}", err);
    }
    assert!(errors.is_empty());

    if let TypeInfo::Enum {
        name,
        variant_types,
        ..
    } = type_engine.look_up_type_id(ty_1)
    {
        assert_eq!(name.as_str(), "Result");
        assert!(matches!(
            type_engine.look_up_type_id(variant_types[0].type_id),
            TypeInfo::Boolean
        ));
    } else {
        panic!()
    }
}

#[test]
fn basic_numeric_unknown() {
    use crate::declaration_engine::DeclarationEngine;
    let type_engine = TypeEngine::default();
    let declaration_engine = DeclarationEngine::default();

    let sp = Span::dummy();
    // numerics
    let id = type_engine.insert_type(&declaration_engine, TypeInfo::Numeric);
    let id2 = type_engine.insert_type(
        &declaration_engine,
        TypeInfo::UnsignedInteger(IntegerBits::Eight),
    );

    // Unify them together...
    let (_, errors) = type_engine.unify(&declaration_engine, id, id2, &sp, "", None);
    assert!(errors.is_empty());

    assert!(matches!(
        type_engine.to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}

#[test]
fn unify_numerics() {
    use crate::declaration_engine::DeclarationEngine;
    let type_engine = TypeEngine::default();
    let declaration_engine = DeclarationEngine::default();
    let sp = Span::dummy();

    // numerics
    let id = type_engine.insert_type(&declaration_engine, TypeInfo::Numeric);
    let id2 = type_engine.insert_type(
        &declaration_engine,
        TypeInfo::UnsignedInteger(IntegerBits::Eight),
    );

    // Unify them together...
    let (_, errors) = type_engine.unify(&declaration_engine, id2, id, &sp, "", None);
    assert!(errors.is_empty());

    assert!(matches!(
        type_engine.to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}

#[test]
fn unify_numerics_2() {
    use crate::declaration_engine::DeclarationEngine;
    let type_engine = TypeEngine::default();
    let declaration_engine = DeclarationEngine::default();
    let sp = Span::dummy();

    // numerics
    let id = type_engine.insert_type(&declaration_engine, TypeInfo::Numeric);
    let id2 = type_engine.insert_type(
        &declaration_engine,
        TypeInfo::UnsignedInteger(IntegerBits::Eight),
    );

    // Unify them together...
    let (_, errors) = type_engine.unify(&declaration_engine, id, id2, &sp, "", None);
    assert!(errors.is_empty());

    assert!(matches!(
        type_engine.to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}
