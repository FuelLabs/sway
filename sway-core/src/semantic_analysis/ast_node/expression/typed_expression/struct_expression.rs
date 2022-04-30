use sway_types::Span;

use crate::{
    error::{err, ok},
    semantic_analysis::TypedStructDeclaration,
    type_engine::{look_up_type_id, TypeId},
    CallPath, CompileError, CompileResult, NamespaceRef, NamespaceWrapper, TypeArgument,
};

pub(crate) fn monomorphize_with_type_arguments(
    call_path: CallPath,
    decl: TypedStructDeclaration,
    type_arguments: Vec<TypeArgument>,
    namespace: NamespaceRef,
    self_type: TypeId,
) -> CompileResult<TypedStructDeclaration> {
    let mut warnings = vec![];
    let mut errors = vec![];
    match (decl.type_parameters.is_empty(), type_arguments.is_empty()) {
        (true, true) => ok(decl, warnings, errors),
        (true, false) => {
            let type_arguments_span = type_arguments
                .iter()
                .map(|x| x.span.clone())
                .reduce(Span::join)
                .unwrap_or_else(|| call_path.suffix.span().clone());
            errors.push(CompileError::DoesNotTakeTypeArguments {
                name: call_path.suffix,
                span: type_arguments_span,
            });
            err(warnings, errors)
        }
        _ => {
            let mut type_arguments = type_arguments;
            for type_argument in type_arguments.iter_mut() {
                type_argument.type_id = check!(
                    namespace.resolve_type_with_self(
                        look_up_type_id(type_argument.type_id),
                        self_type,
                        type_argument.span.clone(),
                        true,
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
            decl.monomorphize(&namespace, &type_arguments, Some(self_type))
        }
    }
}
