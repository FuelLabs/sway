use sway_types::{Span, Spanned};

use crate::{
    error::{err, ok},
    namespace::Path,
    semantic_analysis::EnforceTypeArguments,
    type_engine::{insert_type, look_up_type_id},
    CompileError, CompileResult, Namespace, TypeInfo,
};

use super::{type_argument, type_parameter, unify_with_self, TypeArgument, TypeId, TypeParameter};

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

impl TypeBinding<TypeInfo> {
    pub(crate) fn type_check(
        self,
        module_path: &Path,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];
        println!(
            "{}",
            module_path
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("::")
        );
        check!(
            namespace.root().check_submodule(&module_path),
            return err(warnings, errors),
            warnings,
            errors
        );
        let type_id = check!(
            namespace.root.resolve_type_with_self(
                insert_type(self.inner),
                self_type,
                &self.span(),
                EnforceTypeArguments::No,
                &module_path
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );
        let type_id = match look_up_type_id(type_id) {
            TypeInfo::Enum {
                name,
                type_parameters,
                variant_types,
            } => {
                check!()
            }
            TypeInfo::Struct {
                name,
                type_parameters,
                fields,
            } => todo!(),
            type_info => {
                errors.push(CompileError::DoesNotTakeTypeArguments {
                    name: type_info.to_string(),
                    span: self.span(),
                });
                type_id
            }
        };
        ok(type_id, warnings, errors)
    }
}

fn unify_type_parameters_and_type_arguments(
    type_parameters: &[TypeParameter],
    type_arguments: &[TypeArgument],
    self_type: TypeId,
    span: &Span,
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // check to see if we got the expected number of type arguments
    if type_parameters.len() != type_arguments.len() {
        errors.push(CompileError::IncorrectNumberOfTypeArguments {
            given: type_arguments.len(),
            expected: type_parameters.len(),
            span: span.clone(),
        });
        return err(warnings, errors);
    }

    // unify the type parameters and the type arguments
    for (type_parameter, type_argument) in type_parameters.into_iter().zip(type_arguments.iter()) {
        let (mut new_warnings, new_errors) = unify_with_self(
            type_parameter.type_id,
            type_argument.type_id,
            self_type,
            &type_parameter.span(),
            "cannot unify the type argument with the type parameter",
        );
        warnings.append(&mut new_warnings);
        errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
    }

    if errors.is_empty() {
        ok((), warnings, errors)
    } else {
        err(warnings, errors)
    }
}
