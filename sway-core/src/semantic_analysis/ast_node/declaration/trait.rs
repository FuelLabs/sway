use sway_error::{
    error::CompileError,
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_upper_camel_case, Spanned};

use crate::{
    declaration_engine::*,
    error::*,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::{Mode, TypeCheckContext},
    type_system::*,
};

impl ty::TyTraitDeclaration {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        trait_decl: TraitDeclaration,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let TraitDeclaration {
            name,
            type_parameters,
            attributes,
            interface_surface,
            methods,
            supertraits,
            visibility,
            span,
        } = trait_decl;

        if !is_upper_camel_case(name.as_str()) {
            warnings.push(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonClassCaseTraitName { name: name.clone() },
            })
        }

        // A temporary namespace for checking within the trait's scope.
        let self_type = insert_type(TypeInfo::SelfType);
        let mut trait_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut trait_namespace).with_self_type(self_type);

        // type check the type parameters, which will insert them into the namespace
        let mut new_type_parameters = vec![];
        for type_parameter in type_parameters.into_iter() {
            new_type_parameters.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_parameter),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // type check the interface surface
        let mut new_interface_surface = vec![];
        let mut dummy_interface_surface = vec![];
        for method in interface_surface.into_iter() {
            let method = check!(
                ty::TyTraitFn::type_check(ctx.by_ref(), method),
                return err(warnings, errors),
                warnings,
                errors
            );
            dummy_interface_surface.push(de_insert_function(
                method.clone().to_dummy_func(Mode::NonAbi),
            ));
            new_interface_surface.push(de_insert_trait_fn(method));
        }

        // Recursively handle supertraits: make their interfaces and methods available to this trait
        check!(
            handle_supertraits(ctx.by_ref(), &supertraits),
            return err(warnings, errors),
            warnings,
            errors
        );

        // insert placeholder functions representing the interface surface
        // to allow methods to use those functions
        check!(
            ctx.namespace.insert_trait_implementation(
                CallPath {
                    prefixes: vec![],
                    suffix: name.clone(),
                    is_absolute: false,
                },
                new_type_parameters
                    .iter()
                    .map(|type_param| TypeArgument {
                        type_id: type_param.type_id,
                        initial_type_id: type_param.initial_type_id,
                        span: type_param.name_ident.span(),
                    })
                    .collect(),
                self_type,
                &dummy_interface_surface,
                &span,
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the methods
        let mut new_methods = vec![];
        for method in methods.into_iter() {
            let method = check!(
                ty::TyFunctionDeclaration::type_check(ctx.by_ref(), method.clone(), true),
                ty::TyFunctionDeclaration::error(method),
                warnings,
                errors
            );
            new_methods.push(de_insert_function(method));
        }

        let typed_trait_decl = ty::TyTraitDeclaration {
            name,
            type_parameters: new_type_parameters,
            interface_surface: new_interface_surface,
            methods: new_methods,
            supertraits,
            visibility,
            attributes,
            span,
        };
        ok(typed_trait_decl, warnings, errors)
    }
}

/// Recursively handle supertraits by adding all their interfaces and methods to some namespace
/// which is meant to be the namespace of the subtrait in question
fn handle_supertraits(mut ctx: TypeCheckContext, supertraits: &[Supertrait]) -> CompileResult<()> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let self_type = ctx.self_type();

    for supertrait in supertraits.iter() {
        match ctx
            .namespace
            .resolve_call_path(&supertrait.name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
                let ty::TyTraitDeclaration {
                    ref interface_surface,
                    ref methods,
                    ref supertraits,
                    ref name,
                    ref type_parameters,
                    ..
                } = check!(
                    CompileResult::from(de_get_trait(decl_id.clone(), &supertrait.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // Retrieve the interface surface for this trait.
                let mut dummy_interface_surface = vec![];
                for decl_id in interface_surface.iter() {
                    let method = check!(
                        CompileResult::from(de_get_trait_fn(decl_id.clone(), &name.span())),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    dummy_interface_surface
                        .push(de_insert_function(method.to_dummy_func(Mode::NonAbi)));
                }

                // Transform the trait type parameters defined at the trait
                // declaration into type arguments for use in the trait map.
                let type_params_as_type_args = type_parameters
                    .iter()
                    .map(|type_param| TypeArgument {
                        type_id: type_param.type_id,
                        initial_type_id: type_param.initial_type_id,
                        span: type_param.name_ident.span(),
                    })
                    .collect::<Vec<_>>();

                // Insert the interface surface methods of the supertrait into
                // the namespace. Specifically do not check for conflicting
                // definitions because this is just a temporary namespace for
                // type checking and these are not actual impl blocks.
                ctx.namespace.insert_trait_implementation(
                    supertrait.name.clone(),
                    type_params_as_type_args.clone(),
                    self_type,
                    &dummy_interface_surface,
                    &supertrait.name.span(),
                );

                // Insert the trait methods of the supertrait into the
                // namespace. Specifically do not check for conflicting
                // definitions because this is just a temporary namespace for
                // type checking and these are not actual impl blocks.
                ctx.namespace.insert_trait_implementation(
                    supertrait.name.clone(),
                    type_params_as_type_args,
                    self_type,
                    methods,
                    &supertrait.name.span(),
                );

                // Recurse to insert versions of interfaces and methods of the
                // *super* supertraits.
                check!(
                    handle_supertraits(ctx.by_ref(), supertraits),
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
