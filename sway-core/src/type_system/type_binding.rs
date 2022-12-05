use sway_types::{Span, Spanned};

use crate::{
    declaration_engine::declaration_engine::*,
    error::*,
    language::{ty, CallPath},
    semantic_analysis::TypeCheckContext,
    type_system::EnforceTypeArguments,
    CreateTypeId, TypeInfo,
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

        let type_engine = ctx.type_engine;

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
        let type_id = check!(
            ctx.resolve_type_with_self(
                type_engine.insert_type(type_info),
                &type_info_span,
                EnforceTypeArguments::No,
                Some(&type_info_prefix)
            ),
            type_engine.insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );

        ok(type_id, warnings, errors)
    }
}

impl TypeBinding<CallPath> {
    pub(crate) fn type_check_with_ident(
        &mut self,
        mut ctx: TypeCheckContext,
    ) -> CompileResult<ty::TyDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let engines = ctx.engines();

        // grab the declaration
        let unknown_decl = check!(
            ctx.namespace.resolve_call_path(&self.inner).cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );

        // replace the self types inside of the type arguments
        for type_argument in self.type_arguments.iter_mut() {
            type_argument.replace_self_type(engines, ctx.self_type());
            type_argument.type_id = check!(
                ctx.resolve_type_without_self(type_argument.type_id, &type_argument.span, None),
                type_engine.insert_type(TypeInfo::ErrorRecovery),
                warnings,
                errors
            );
        }

        // monomorphize the declaration, if needed
        let new_decl = match unknown_decl {
            ty::TyDeclaration::FunctionDeclaration(original_id) => {
                // get the copy from the declaration engine
                let mut new_copy = check!(
                    CompileResult::from(de_get_function(original_id.clone(), &self.span())),
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
                let new_id = ctx
                    .declaration_engine
                    .insert_function(new_copy)
                    .with_parent(ctx.declaration_engine, original_id);

                ty::TyDeclaration::FunctionDeclaration(new_id)
            }
            ty::TyDeclaration::EnumDeclaration(original_id) => {
                // get the copy from the declaration engine
                let mut new_copy = check!(
                    CompileResult::from(de_get_enum(original_id, &self.span())),
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

                // take any trait methods that apply to this type and copy them to the new type
                ctx.namespace.insert_trait_implementation_for_type(
                    engines,
                    new_copy.create_type_id(type_engine),
                );

                // insert the new copy into the declaration engine
                let new_id = ctx.declaration_engine.insert_enum(new_copy);

                ty::TyDeclaration::EnumDeclaration(new_id)
            }
            ty::TyDeclaration::StructDeclaration(original_id) => {
                // get the copy from the declaration engine
                let mut new_copy = check!(
                    CompileResult::from(de_get_struct(original_id, &self.span())),
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

                // take any trait methods that apply to this type and copy them to the new type
                ctx.namespace.insert_trait_implementation_for_type(
                    engines,
                    new_copy.create_type_id(type_engine),
                );

                // insert the new copy into the declaration engine
                let new_id = ctx.declaration_engine.insert_struct(new_copy);

                ty::TyDeclaration::StructDeclaration(new_id)
            }
            _ => unknown_decl,
        };
        ok(new_decl, warnings, errors)
    }
}
