use derivative::Derivative;
use sway_error::{
    error::CompileError,
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_upper_camel_case, Ident, Spanned};

use crate::{
    declaration_engine::*,
    error::*,
    language::{parsed::*, ty, CallPath, Visibility},
    semantic_analysis::{
        ast_node::{type_check_interface_surface, type_check_trait_methods},
        Mode, TyCodeBlock, TypeCheckContext,
    },
    type_system::*,
    Namespace,
};

use super::{EnforceTypeArguments, TyFunctionParameter, TyTraitFn};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TyTraitDeclaration {
    pub name: Ident,
    pub interface_surface: Vec<TyTraitFn>,
    // NOTE: deriving partialeq and hash on this element may be important in the
    // future, but I am not sure. For now, adding this would 2x the amount of
    // work, so I am just going to exclude it
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) methods: Vec<FunctionDeclaration>,
    pub(crate) supertraits: Vec<Supertrait>,
    pub visibility: Visibility,
}

impl CopyTypes for TyTraitDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.interface_surface
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}

impl TyTraitDeclaration {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        trait_decl: TraitDeclaration,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let name = trait_decl.name.clone();
        if !is_upper_camel_case(name.as_str()) {
            warnings.push(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonClassCaseTraitName { name },
            })
        }

        // type check the interface surface
        let interface_surface = check!(
            type_check_interface_surface(trait_decl.interface_surface.to_vec(), ctx.namespace),
            return err(warnings, errors),
            warnings,
            errors
        );

        // A temporary namespace for checking within the trait's scope.
        let mut trait_namespace = ctx.namespace.clone();
        let ctx = ctx.scoped(&mut trait_namespace);

        // Recursively handle supertraits: make their interfaces and methods available to this trait
        check!(
            handle_supertraits(&trait_decl.supertraits, ctx.namespace),
            return err(warnings, errors),
            warnings,
            errors
        );

        // insert placeholder functions representing the interface surface
        // to allow methods to use those functions
        ctx.namespace.insert_trait_implementation(
            CallPath {
                prefixes: vec![],
                suffix: trait_decl.name.clone(),
                is_absolute: false,
            },
            insert_type(TypeInfo::SelfType),
            interface_surface
                .iter()
                .map(|x| x.to_dummy_func(Mode::NonAbi))
                .collect(),
        );
        // check the methods for errors but throw them away and use vanilla [FunctionDeclaration]s
        let ctx = ctx.with_self_type(insert_type(TypeInfo::SelfType));
        let _methods = check!(
            type_check_trait_methods(ctx, trait_decl.methods.clone()),
            vec![],
            warnings,
            errors
        );
        let typed_trait_decl = TyTraitDeclaration {
            name: trait_decl.name,
            interface_surface,
            methods: trait_decl.methods.to_vec(),
            supertraits: trait_decl.supertraits.to_vec(),
            visibility: trait_decl.visibility,
        };
        ok(typed_trait_decl, warnings, errors)
    }
}

/// Recursively handle supertraits by adding all their interfaces and methods to some namespace
/// which is meant to be the namespace of the subtrait in question
fn handle_supertraits(
    supertraits: &[Supertrait],
    trait_namespace: &mut Namespace,
) -> CompileResult<()> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for supertrait in supertraits.iter() {
        match trait_namespace
            .resolve_call_path(&supertrait.name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
                let TyTraitDeclaration {
                    ref interface_surface,
                    ref methods,
                    ref supertraits,
                    ..
                } = check!(
                    CompileResult::from(de_get_trait(decl_id.clone(), &supertrait.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // insert dummy versions of the interfaces for all of the supertraits
                trait_namespace.insert_trait_implementation(
                    supertrait.name.clone(),
                    insert_type(TypeInfo::SelfType),
                    interface_surface
                        .iter()
                        .map(|x| x.to_dummy_func(Mode::NonAbi))
                        .collect(),
                );

                // insert dummy versions of the methods of all of the supertraits
                let dummy_funcs = check!(
                    convert_trait_methods_to_dummy_funcs(methods, trait_namespace),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                trait_namespace.insert_trait_implementation(
                    supertrait.name.clone(),
                    insert_type(TypeInfo::SelfType),
                    dummy_funcs,
                );

                // Recurse to insert dummy versions of interfaces and methods of the *super*
                // supertraits
                check!(
                    handle_supertraits(supertraits, trait_namespace),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
            Some(ty::TyDeclaration::AbiDeclaration(_)) => {
                errors.push(CompileError::AbiAsSupertrait {
                    span: supertrait.name.span().clone(),
                })
            }
            _ => errors.push(CompileError::TraitNotFound {
                name: supertrait.name.to_string(),
                span: supertrait.name.span(),
            }),
        }
    }

    ok((), warnings, errors)
}

/// Convert a vector of FunctionDeclarations into a vector of [ty::TyFunctionDeclaration]'s where only
/// the parameters and the return types are type checked.
fn convert_trait_methods_to_dummy_funcs(
    methods: &[FunctionDeclaration],
    trait_namespace: &mut Namespace,
) -> CompileResult<Vec<ty::TyFunctionDeclaration>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut dummy_funcs = vec![];
    for method in methods.iter() {
        let FunctionDeclaration {
            name,
            parameters,
            return_type,
            return_type_span,
            ..
        } = method;

        // type check the parameters
        let mut typed_parameters = vec![];
        for param in parameters.iter() {
            typed_parameters.push(check!(
                TyFunctionParameter::type_check_interface_parameter(trait_namespace, param.clone()),
                continue,
                warnings,
                errors
            ));
        }

        // type check the return type
        let initial_return_type = insert_type(return_type.clone());
        let return_type = check!(
            trait_namespace.resolve_type_with_self(
                initial_return_type,
                insert_type(TypeInfo::SelfType),
                return_type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        dummy_funcs.push(ty::TyFunctionDeclaration {
            purity: Default::default(),
            name: name.clone(),
            body: TyCodeBlock { contents: vec![] },
            parameters: typed_parameters,
            attributes: method.attributes.clone(),
            span: name.span(),
            return_type,
            initial_return_type,
            return_type_span: return_type_span.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: false,
        });
    }
    if errors.is_empty() {
        ok(dummy_funcs, warnings, errors)
    } else {
        err(warnings, errors)
    }
}
