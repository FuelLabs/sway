use sway_types::{Span, Spanned};

use crate::{
    error::{err, ok},
    semantic_analysis::TypeCheckContext,
    CallPath, CompileResult, TypedDeclaration,
};

use super::{EnforceTypeArguments, ReplaceSelfType, TypeArgument};

/// A `TypeBinding` is the result of using turbofish to bind types to
/// generic parameters.
///
/// For example:
///
/// ```ignore
/// let data = Data::<bool> {
///   value: true
/// };
/// ```
///
/// Would produce the type binding (in pseudocode):
///
/// ```ignore
/// TypeBinding {
///     inner: CallPath(["Data"]),
///     type_arguments: [bool]
/// }
/// ```
///
/// ---
///
/// Further:
///
/// ```ignore
/// struct Data<T> {
///   value: T
/// }
///
/// let data1 = Data {
///   value: true
/// };
///
/// let data2 = Data::<bool> {
///   value: true
/// };
///
/// let data3: Data<bool> = Data {
///   value: true
/// };
///
/// let data4: Data<bool> = Data::<bool> {
///   value: true
/// };
/// ```
///
/// Each of these 4 examples generates a valid struct expression for `Data`
/// and passes type checking. But each does so in a unique way:
/// - `data1` has no type ascription and no type arguments in the `TypeBinding`,
///     so both are inferred from the value passed to `value`
/// - `data2` has no type ascription but does have type arguments in the
///     `TypeBinding`, so the type ascription and type of the value passed to
///     `value` are both unified to the `TypeBinding`
/// - `data3` has a type ascription but no type arguments in the `TypeBinding`,
///     so the type arguments in the `TypeBinding` and the type of the value
///     passed to `value` are both unified to the type ascription
/// - `data4` has a type ascription and has type arguments in the `TypeBinding`,
///     so, with the type from the value passed to `value`, all three are unified
///     together
#[derive(Debug, Clone)]
pub struct TypeBinding<T> {
    pub inner: T,
    pub type_arguments: Vec<TypeArgument>,
    pub span: Span,
}

impl<T> Spanned for TypeBinding<T> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<T> TypeBinding<T> {
    pub(crate) fn new_with_empty_type_arguments(inner: T, span: Span) -> TypeBinding<T> {
        TypeBinding {
            inner,
            type_arguments: vec![],
            span,
        }
    }
}

impl TypeBinding<CallPath> {
    pub(crate) fn type_check(
        &mut self,
        ctx: &mut TypeCheckContext,
    ) -> CompileResult<TypedDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        for type_argument in self.type_arguments.iter_mut() {
            type_argument.replace_self_type(ctx.self_type());
        }
        let mut unknown_decl = check!(
            ctx.namespace.resolve_call_path(&self.inner).cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );
        match unknown_decl {
            TypedDeclaration::FunctionDeclaration(ref mut decl) => {
                check!(
                    ctx.monomorphize(
                        decl,
                        &mut self.type_arguments,
                        EnforceTypeArguments::No,
                        &self.span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            TypedDeclaration::EnumDeclaration(ref mut decl) => {
                check!(
                    ctx.monomorphize(
                        decl,
                        &mut self.type_arguments,
                        EnforceTypeArguments::No,
                        &self.span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            TypedDeclaration::StructDeclaration(ref mut decl) => {
                check!(
                    ctx.monomorphize(
                        decl,
                        &mut self.type_arguments,
                        EnforceTypeArguments::No,
                        &self.span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            _ => {}
        }
        ok(unknown_decl, warnings, errors)
    }
}
