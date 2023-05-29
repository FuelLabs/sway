mod ast_elements;
mod engine;
mod id;
mod info;
mod priv_prelude;
mod substitute;
mod unify;

pub use priv_prelude::*;

#[cfg(test)]
use crate::{language::ty::TyEnumDecl, transform::AttributesMap};

#[cfg(test)]
use sway_types::{integer_bits::IntegerBits, Span};

#[test]
fn generic_enum_resolution() {
    use crate::{
        decl_engine::DeclEngineInsert, language::ty, span::Span, transform, Engines, Ident,
    };

    let engines = Engines::default();

    let sp = Span::dummy();
    let generic_name = Ident::new_with_override("T".into(), sp.clone());
    let a_name = Ident::new_with_override("a".into(), sp.clone());
    let result_name = Ident::new_with_override("Result".into(), sp.clone());

    /*
    Result<_> {
        a: _
    }
    */
    let generic_type = engines.te().insert(
        &engines,
        TypeInfo::UnknownGeneric {
            name: generic_name.clone(),
            trait_constraints: VecSet(Vec::new()),
        },
    );
    let placeholder_type = engines.te().insert(
        &engines,
        TypeInfo::Placeholder(TypeParameter {
            type_id: generic_type,
            initial_type_id: generic_type,
            name_ident: generic_name.clone(),
            trait_constraints: vec![],
            trait_constraints_span: sp.clone(),
            is_from_parent: false,
        }),
    );
    let placeholder_type_param = TypeParameter {
        type_id: placeholder_type,
        initial_type_id: placeholder_type,
        name_ident: generic_name.clone(),
        trait_constraints: vec![],
        trait_constraints_span: sp.clone(),
        is_from_parent: false,
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

    let decl_ref_1 = engines.de().insert(TyEnumDecl {
        call_path: result_name.clone().into(),
        type_parameters: vec![placeholder_type_param],
        variants: variant_types,
        span: sp.clone(),
        visibility: crate::language::Visibility::Public,
        attributes: AttributesMap::default(),
    });
    let ty_1 = engines.te().insert(&engines, TypeInfo::Enum(decl_ref_1));

    /*
    Result<bool> {
        a: bool
    }
    */
    let boolean_type = engines.te().insert(&engines, TypeInfo::Boolean);
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
        is_from_parent: false,
    };
    let decl_ref_2 = engines.de().insert(TyEnumDecl {
        call_path: result_name.into(),
        type_parameters: vec![type_param],
        variants: variant_types.clone(),
        span: sp.clone(),
        visibility: crate::language::Visibility::Public,
        attributes: AttributesMap::default(),
    });
    let ty_2 = engines.te().insert(&engines, TypeInfo::Enum(decl_ref_2));

    // Unify them together...
    let (_, errors) = engines.te().unify(&engines, ty_1, ty_2, &sp, "", None);
    assert!(errors.is_empty());

    if let TypeInfo::Enum(decl_ref_1) = engines.te().get(ty_1) {
        let decl = engines.de().get_enum(&decl_ref_1);
        assert_eq!(decl.call_path.suffix.as_str(), "Result");
        assert!(matches!(
            engines.te().get(variant_types[0].type_argument.type_id),
            TypeInfo::Boolean
        ));
    } else {
        panic!()
    }
}

#[test]
fn basic_numeric_unknown() {
    use crate::decl_engine::DeclEngine;
    use crate::Engines;
    let engines = Engines::default();

    let sp = Span::dummy();
    // numerics
    let id = engines.te().insert(&engines, TypeInfo::Numeric);
    let id2 = engines
        .te()
        .insert(&engines, TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engines.te().unify(&engines, id, id2, &sp, "", None);
    assert!(errors.is_empty());

    assert!(matches!(
        engines.te().to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}

#[test]
fn unify_numerics() {
    use crate::decl_engine::DeclEngine;
    use crate::Engines;
    let engines = Engines::default();
    let sp = Span::dummy();

    // numerics
    let id = engines.te().insert(&engines, TypeInfo::Numeric);
    let id2 = engines
        .te()
        .insert(&engines, TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = engines.te().unify(&engines, id2, id, &sp, "", None);
    assert!(errors.is_empty());

    assert!(matches!(
        engines.te().to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}

#[test]
fn unify_numerics_2() {
    use crate::{decl_engine::DeclEngine, Engines};
    let engines = Engines::default();
    let type_engine = engines.te();
    let sp = Span::dummy();

    // numerics
    let id = type_engine.insert(&engines, TypeInfo::Numeric);
    let id2 = type_engine.insert(&engines, TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    let (_, errors) = type_engine.unify(&engines, id, id2, &sp, "", None);
    assert!(errors.is_empty());

    assert!(matches!(
        type_engine.to_typeinfo(id, &Span::dummy()).unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::Eight)
    ));
}
