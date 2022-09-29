use sway_types::{Span, Spanned};

use crate::{
    declaration_engine::declaration_engine::*,
    error::{err, ok},
    semantic_analysis::TypeCheckContext,
    type_system::{insert_type, EnforceTypeArguments},
    CallPath, CompileResult, TypeInfo, TypedDeclaration,
};

use super::{ReplaceSelfType, TypeArgument, TypeId};

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

impl TypeBinding<CallPath<(TypeInfo, Span)>> {
    pub(crate) fn type_check_with_type_info(
        &self,
        ctx: &mut TypeCheckContext,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let (type_info, type_info_span) = self.inner.suffix.clone();

        // find the module that the symbol is in
        let type_info_prefix = ctx.namespace.find_module_path(&self.inner.prefixes);
        check!(
            ctx.namespace.root().check_submodule(&type_info_prefix),
            return err(warnings, errors),
            warnings,
            errors
        );

        // create the type info object
        let type_info = check!(
            type_info.apply_type_arguments(self.type_arguments.clone(), &type_info_span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // resolve the type of the type info object
        let type_id = insert_type(type_info);
        append!(
            ctx.resolve_type_with_self(
                type_id,
                &type_info_span,
                EnforceTypeArguments::No,
                Some(&type_info_prefix)
            ),
            warnings,
            errors
        );

        ok(type_id, warnings, errors)
    }
}

impl TypeBinding<CallPath> {
    pub(crate) fn type_check_with_ident(
        &mut self,
        ctx: &TypeCheckContext,
    ) -> CompileResult<TypedDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // grab the declaration
        let unknown_decl = check!(
            ctx.namespace.resolve_call_path(&self.inner).cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );

        // replace the self types inside of the type arguments
        for type_argument in self.type_arguments.iter_mut() {
            type_argument.replace_self_type(ctx.self_type());
        }

        // monomorphize the declaration, if needed
        let new_decl = match unknown_decl {
            TypedDeclaration::FunctionDeclaration(original_id) => {
                // get the copy from the declaration engine
                let mut new_copy = check!(
                    CompileResult::from(de_get_function(original_id.clone(), &self.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                if new_copy.name.as_str() == "second_if" || new_copy.name.as_str() == "third_if" {
                    println!("type binding before: {}", new_copy);
                }

                // monomorphize the copy, in place
                check!(
                    ctx.monomorphize(
                        &mut new_copy,
                        &mut self.type_arguments,
                        EnforceTypeArguments::No,
                        &self.span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                if new_copy.name.as_str() == "second_if" || new_copy.name.as_str() == "third_if" {
                    println!(" type binding after: {}", new_copy);
                }

                // insert the new copy into the declaration engine
                let new_id = de_insert_function(new_copy);

                // add the new copy as a monomorphized copy of the original id
                de_add_monomorphized_copy(original_id, new_id.clone());

                TypedDeclaration::FunctionDeclaration(new_id)
            }
            TypedDeclaration::EnumDeclaration(original_id) => {
                // get the copy from the declaration engine
                let mut new_copy = check!(
                    CompileResult::from(de_get_enum(original_id.clone(), &self.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // monomorphize the copy, in place
                check!(
                    ctx.monomorphize(
                        &mut new_copy,
                        &mut self.type_arguments,
                        EnforceTypeArguments::No,
                        &self.span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // insert the new copy into the declaration engine
                let new_id = de_insert_enum(new_copy);

                // add the new copy as a monomorphized copy of the original id
                de_add_monomorphized_copy(original_id, new_id.clone());

                TypedDeclaration::EnumDeclaration(new_id)
            }
            TypedDeclaration::StructDeclaration(original_id) => {
                // get the copy from the declaration engine
                let mut new_copy = check!(
                    CompileResult::from(de_get_struct(original_id.clone(), &self.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // monomorphize the copy, in place
                check!(
                    ctx.monomorphize(
                        &mut new_copy,
                        &mut self.type_arguments,
                        EnforceTypeArguments::No,
                        &self.span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // insert the new copy into the declaration engine
                let new_id = de_insert_struct(new_copy);

                // add the new copy as a monomorphized copy of the original id
                de_add_monomorphized_copy(original_id, new_id.clone());

                TypedDeclaration::StructDeclaration(new_id)
            }
            _ => unknown_decl,
        };
        ok(new_decl, warnings, errors)
    }
}
