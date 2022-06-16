use sway_types::{Ident, Span, Spanned};

use crate::{
    error::*, namespace::*, type_engine::*, CompileError, CompileResult, TypeArgument, TypeInfo,
    TypeParameter,
};

/// This type is used to denote if, during monomorphization, the compiler
/// should enforce that type arguments be provided. An example of that
/// might be this:
///
/// ```ignore
/// struct Point<T> {
///   x: u64,
///   y: u64
/// }
///
/// fn add<T>(p1: Point<T>, p2: Point<T>) -> Point<T> {
///   Point {
///     x: p1.x + p2.x,
///     y: p1.y + p2.y
///   }
/// }
/// ```
///
/// `EnforeTypeArguments` would require that the type annotations
/// for `p1` and `p2` contain `<...>`. This is to avoid ambiguous definitions:
///
/// ```ignore
/// fn add(p1: Point, p2: Point) -> Point {
///   Point {
///     x: p1.x + p2.x,
///     y: p1.y + p2.y
///   }
/// }
/// ```
#[derive(Clone, Copy)]
pub(crate) enum EnforceTypeArguments {
    Yes,
    No,
}

pub(crate) trait Monomorphize {
    type Output;

    fn monomorphize(
        self,
        type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        call_site_span: Option<&Span>,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<Self::Output>;
}

impl<T> Monomorphize for T
where
    T: MonomorphizeHelper + Spanned + ResolveTypes,
{
    type Output = T;

    fn monomorphize(
        mut self,
        mut type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        call_site_span: Option<&Span>,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<Self::Output> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match (self.type_parameters().is_empty(), type_arguments.is_empty()) {
            (true, true) => ok(self, warnings, errors),
            (false, true) => {
                if let EnforceTypeArguments::Yes = enforce_type_arguments {
                    let name_span = self.name().span();
                    errors.push(CompileError::NeedsTypeArguments {
                        name: self.name().clone(),
                        span: call_site_span.unwrap_or(&name_span).clone(),
                    });
                    return err(warnings, errors);
                }
                check!(
                    self.resolve_types(
                        type_arguments,
                        enforce_type_arguments,
                        namespace,
                        module_path
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ok(self, warnings, errors)
            }
            (true, false) => {
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(Span::join)
                    .unwrap_or_else(|| self.span());
                errors.push(CompileError::DoesNotTakeTypeArguments {
                    name: self.name().clone(),
                    span: type_arguments_span,
                });
                err(warnings, errors)
            }
            (false, false) => {
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(Span::join)
                    .unwrap_or_else(|| self.span());
                if self.type_parameters().len() != type_arguments.len() {
                    errors.push(CompileError::IncorrectNumberOfTypeArguments {
                        given: type_arguments.len(),
                        expected: self.type_parameters().len(),
                        span: type_arguments_span,
                    });
                    return err(warnings, errors);
                }
                for type_argument in type_arguments.iter_mut() {
                    type_argument.type_id = check!(
                        namespace.resolve_type(
                            type_argument.type_id,
                            &type_argument.span,
                            enforce_type_arguments,
                            module_path
                        ),
                        insert_type(TypeInfo::ErrorRecovery),
                        warnings,
                        errors
                    );
                }
                check!(
                    self.resolve_types(
                        type_arguments,
                        enforce_type_arguments,
                        namespace,
                        module_path
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ok(self, warnings, errors)
            }
        }
    }
}

pub(crate) trait MonomorphizeHelper {
    fn type_parameters(&self) -> &[TypeParameter];
    fn name(&self) -> &Ident;
}
