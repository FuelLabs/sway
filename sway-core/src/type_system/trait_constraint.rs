use sway_error::error::CompileError;
use sway_types::Spanned;

use crate::{
    declaration_engine::{de_get_trait, de_get_trait_fn, de_insert_function},
    error::*,
    language::{ty, CallPath},
    semantic_analysis::{Mode, TypeCheckContext},
    type_system::*,
    CompileResult,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct TraitConstraint {
    pub(crate) trait_name: CallPath,
    pub(crate) type_arguments: Vec<TypeArgument>,
}

impl Spanned for TraitConstraint {
    fn span(&self) -> sway_types::Span {
        self.trait_name.span()
    }
}

impl TraitConstraint {
    pub(crate) fn type_check(
        &mut self,
        mut ctx: TypeCheckContext,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        for type_argument in self.type_arguments.iter_mut() {
            type_argument.type_id = check!(
                ctx.resolve_type_without_self(
                    type_argument.type_id,
                    &type_argument.span,
                    None
                ),
                insert_type(TypeInfo::ErrorRecovery),
                warnings,
                errors
            );
        }

        ok((), warnings, errors)
    }

    pub(crate) fn insert_into_namespace(
        &mut self,
        type_id: TypeId,
        mut ctx: TypeCheckContext,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        match ctx
            .namespace
            .resolve_call_path(&self.trait_name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
                let ty::TyTraitDeclaration {
                    interface_surface,
                    methods,
                    name,
                    type_parameters,
                    ..
                } = check!(
                    CompileResult::from(de_get_trait(decl_id, &self.trait_name.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // Retrieve the trait methods for this trait. Transform them
                // into the correct typing for this impl block by using the
                // type parameters from the original trait declaration and the
                // type arguments of the trait constraint.
                let mut trait_methods = methods;
                let type_mapping = TypeMapping::from_type_parameters_and_type_arguments(
                    type_parameters
                        .iter()
                        .map(|type_param| type_param.type_id)
                        .collect(),
                    self.type_arguments
                        .iter()
                        .map(|type_arg| type_arg.type_id)
                        .collect(),
                );
                for decl_id in interface_surface.into_iter() {
                    let mut method = check!(
                        CompileResult::from(de_get_trait_fn(decl_id.clone(), &name.span())),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    method.replace_self_type(type_id);
                    method.copy_types(&type_mapping);
                    trait_methods.push(
                        de_insert_function(method.to_dummy_func(Mode::NonAbi)).with_parent(decl_id),
                    );
                }

                // Insert the methods of the supertrait into the namespace.
                // Specifically do not check for conflicting definitions because
                // this is just a temporary namespace for type checking and
                // these are not actual impl blocks.
                ctx.namespace.insert_trait_implementation(
                    self.trait_name.clone(),
                    self.type_arguments.clone(),
                    type_id,
                    &trait_methods,
                    &self.trait_name.span(),
                    false,
                );
            }
            Some(ty::TyDeclaration::AbiDeclaration(_)) => {
                errors.push(CompileError::AbiAsSupertrait {
                    span: self.trait_name.span(),
                })
            }
            _ => errors.push(CompileError::TraitNotFound {
                name: self.trait_name.to_string(),
                span: self.trait_name.span(),
            }),
        }

        ok((), warnings, errors)
    }
}
