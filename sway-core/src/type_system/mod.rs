mod binding;
mod collect_types_metadata;
mod create_type_id;
mod engine;
mod id;
mod info;
mod length;
mod occurs_check;
mod replace_self_type;
mod substitute;
mod trait_constraint;
mod type_argument;
mod type_parameter;
mod unconstrained_type_parameters;
mod unify;
mod unify_check;

pub(crate) use binding::*;
pub(crate) use collect_types_metadata::*;
pub(crate) use create_type_id::*;
pub use engine::*;
pub use id::*;
pub use info::*;
pub use length::*;
use occurs_check::*;
pub(crate) use replace_self_type::*;
pub(crate) use substitute::*;
pub use trait_constraint::*;
pub use type_argument::*;
pub use type_parameter::*;
pub(crate) use unconstrained_type_parameters::*;

use crate::error::*;
#[cfg(test)]
use crate::{
    decl_engine::DeclEngineInsert, language::ty::TyEnumDeclaration, transform::AttributesMap,
};
use std::fmt::Debug;

#[cfg(test)]
use sway_types::{integer_bits::IntegerBits, Span};

#[test]
fn generic_enum_resolution() {
    use crate::{decl_engine::DeclEngine, language::ty, span::Span, transform, Ident};
    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();

    let sp = Span::dummy();
    let generic_name = Ident::new_with_override("T".into(), sp.clone());
    let a_name = Ident::new_with_override("a".into(), sp.clone());
    let result_name = Ident::new_with_override("Result".into(), sp.clone());

    /*
    Result<_> {
        a: _
    }
    */
    let generic_type = type_engine.insert(
        &decl_engine,
        TypeInfo::UnknownGeneric {
            name: generic_name.clone(),
            trait_constraints: VecSet(Vec::new()),
        },
    );
    let placeholder_type = type_engine.insert(
        &decl_engine,
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
        type_argument: TypeArgument {
            type_id: placeholder_type,
            initial_type_id: placeholder_type,
            span: sp.clone(),
            call_path_tree: None,
        },
        span: sp.clone(),
        attributes: transform::AttributesMap::default(),
    }];

    let decl_ref_1 = decl_engine.insert(TyEnumDeclaration {
        call_path: result_name.clone().into(),
        type_parameters: vec![placeholder_type_param],
        variants: variant_types,
        span: sp.clone(),
        visibility: crate::language::Visibility::Public,
        attributes: AttributesMap::default(),
    });
    let ty_1 = type_engine.insert(&decl_engine, TypeInfo::Enum(decl_ref_1));

    /*
    Result<bool> {
        a: bool
    }
    */
    let boolean_type = type_engine.insert(&decl_engine, TypeInfo::Boolean);
    let variant_types = vec![ty::TyEnumVariant {
        name: a_name,
        tag: 0,
        type_argument: TypeArgument {
            type_id: boolean_type,
            initial_type_id: boolean_type,
            span: sp.clone(),
            call_path_tree: None,
        },
        span: sp.clone(),
        attributes: transform::AttributesMap::default(),
    }];
    let type_param = TypeParameter {
        type_id: boolean_type,
        initial_type_id: boolean_type,
        name_ident: generic_name,
        trait_constraints: vec![],
        trait_constraints_span: sp.clone(),
    };
    let decl_ref_2 = decl_engine.insert(TyEnumDeclaration {
        call_path: result_name.into(),
        type_parameters: vec![type_param],
        variants: variant_types.clone(),
        span: sp.clone(),
        visibility: crate::language::Visibility::Public,
        attributes: AttributesMap::default(),
    });
    let ty_2 = type_engine.insert(&decl_engine, TypeInfo::Enum(decl_ref_2));

    // Unify them together...
    let (_, errors) = type_engine.unify(&decl_engine, ty_1, ty_2, &sp, "", None);
    assert!(errors.is_empty());

    if let TypeInfo::Enum(decl_ref_1) = type_engine.get(ty_1) {
        let decl = decl_engine.get_enum(&decl_ref_1);
        assert_eq!(decl.call_path.suffix.as_str(), "Result");
        assert!(matches!(
            type_engine.get(variant_types[0].type_argument.type_id),
            TypeInfo::Boolean
        ));
    } else {
        panic!()
    }
}

#[test]
fn basic_numeric_unknown() {
    use crate::decl_engine::DeclEngine;
    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();

    let sp = Span::dummy();
    // numerics
    let id = type_engine.insert(&decl_engine, TypeInfo::Numeric);
    let id2 = type_engine.insert(&decl_engine, TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = type_engine.unify(&decl_engine, id, id2, &sp, "", None);
    assert!(errors.is_empty());

    assert!(matches!(
        type_engine.to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}

#[test]
fn unify_numerics() {
    use crate::decl_engine::DeclEngine;
    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();
    let sp = Span::dummy();

    // numerics
    let id = type_engine.insert(&decl_engine, TypeInfo::Numeric);
    let id2 = type_engine.insert(&decl_engine, TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = type_engine.unify(&decl_engine, id2, id, &sp, "", None);
    assert!(errors.is_empty());

    assert!(matches!(
        type_engine.to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}

#[test]
fn unify_numerics_2() {
    use crate::decl_engine::DeclEngine;
    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();
    let sp = Span::dummy();

    // numerics
    let id = type_engine.insert(&decl_engine, TypeInfo::Numeric);
    let id2 = type_engine.insert(&decl_engine, TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = type_engine.unify(&decl_engine, id, id2, &sp, "", None);
    assert!(errors.is_empty());

    assert!(matches!(
        type_engine.to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}
