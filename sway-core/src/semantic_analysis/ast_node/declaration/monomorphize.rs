use sway_types::{Ident, Span};

use crate::{
    error::{err, ok},
    semantic_analysis::{insert_type_parameters, CopyTypes, TypeMapping},
    type_engine::{insert_type, look_up_type_id, unify, unify_with_self, TypeId},
    CompileError, CompileResult, NamespaceRef, NamespaceWrapper, TypeArgument, TypeInfo,
    TypeParameter,
};

use super::CreateTypeId;

pub(crate) trait Monomorphize {
    type Output;

    fn monomorphize(
        self,
        type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: bool,
        namespace: &NamespaceRef,
        self_type: Option<TypeId>,
        call_site_span: Option<&Span>,
    ) -> CompileResult<Self::Output>;

    fn monomorphize_inner(
        self,
        type_mapping: &TypeMapping,
        namespace: &NamespaceRef,
    ) -> Self::Output;
}

impl<T> Monomorphize for T
where
    T: CopyTypes + MonomorphizeHelper + CreateTypeId,
{
    type Output = T;

    fn monomorphize(
        self,
        type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: bool,
        namespace: &NamespaceRef,
        self_type: Option<TypeId>,
        call_site_span: Option<&Span>,
    ) -> CompileResult<Self::Output> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match (self.type_parameters().is_empty(), type_arguments.is_empty()) {
            (true, true) => ok(self, warnings, errors),
            (false, true) => {
                if enforce_type_arguments {
                    errors.push(CompileError::NeedsTypeArguments {
                        name: self.name().clone(),
                        span: call_site_span.unwrap_or_else(|| self.name().span()).clone(),
                    });
                    return err(warnings, errors);
                }
                let type_mapping = insert_type_parameters(self.type_parameters());
                let new_decl = self.monomorphize_inner(&type_mapping, namespace);
                ok(new_decl, warnings, errors)
            }
            (true, false) => {
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(Span::join)
                    .unwrap_or_else(|| self.span().clone());
                errors.push(CompileError::DoesNotTakeTypeArguments {
                    name: self.name().clone(),
                    span: type_arguments_span,
                });
                err(warnings, errors)
            }
            (false, false) => {
                let mut type_arguments = type_arguments;
                for type_argument in type_arguments.iter_mut() {
                    let type_id = match self_type {
                        Some(self_type) => namespace.resolve_type_with_self(
                            look_up_type_id(type_argument.type_id),
                            self_type,
                            &type_argument.span,
                            enforce_type_arguments,
                        ),
                        None => namespace
                            .resolve_type_without_self(&look_up_type_id(type_argument.type_id)),
                    };
                    type_argument.type_id = check!(
                        type_id,
                        insert_type(TypeInfo::ErrorRecovery),
                        warnings,
                        errors
                    );
                }
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(Span::join)
                    .unwrap_or_else(|| self.span().clone());
                if self.type_parameters().len() != type_arguments.len() {
                    errors.push(CompileError::IncorrectNumberOfTypeArguments {
                        given: type_arguments.len(),
                        expected: self.type_parameters().len(),
                        span: type_arguments_span,
                    });
                    return err(warnings, errors);
                }
                let type_mapping = insert_type_parameters(self.type_parameters());
                for ((_, interim_type), type_argument) in
                    type_mapping.iter().zip(type_arguments.iter())
                {
                    match self_type {
                        Some(self_type) => {
                            let (mut new_warnings, new_errors) = unify_with_self(
                                *interim_type,
                                type_argument.type_id,
                                self_type,
                                &type_argument.span,
                                "Type argument is not assignable to generic type parameter.",
                            );
                            warnings.append(&mut new_warnings);
                            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
                        }
                        None => {
                            let (mut new_warnings, new_errors) = unify(
                                *interim_type,
                                type_argument.type_id,
                                &type_argument.span,
                                "Type argument is not assignable to generic type parameter.",
                            );
                            warnings.append(&mut new_warnings);
                            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
                        }
                    }
                }
                let new_decl = self.monomorphize_inner(&type_mapping, namespace);
                ok(new_decl, warnings, errors)
            }
        }
    }

    fn monomorphize_inner(self, type_mapping: &TypeMapping, namespace: &NamespaceRef) -> T {
        let old_type_id = self.type_id();
        let mut new_decl = self;
        new_decl.copy_types(type_mapping);
        namespace.copy_methods_to_type(
            look_up_type_id(old_type_id),
            look_up_type_id(new_decl.type_id()),
            type_mapping,
        );
        new_decl
    }
}

pub(crate) trait MonomorphizeHelper {
    fn type_parameters(&self) -> &[TypeParameter];
    fn name(&self) -> &Ident;
    fn span(&self) -> &Span;
}
