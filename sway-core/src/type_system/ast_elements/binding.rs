use sway_types::{Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
    error::*,
    language::{ty, CallPath},
    semantic_analysis::TypeCheckContext,
    type_system::priv_prelude::*,
    Ident,
};

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
    pub type_arguments: TypeArgs,
    pub span: Span,
}

/// A [TypeArgs] contains a `Vec<TypeArgument>` either in the variant `Regular`
/// or in the variant `Prefix`.
///
/// `Regular` variant indicates the type arguments are located after the suffix.
/// `Prefix` variant indicates the type arguments are located between the last
/// prefix and the suffix.
///
/// In the case of an enum we can have either the type parameters in the `Regular`
/// variant, case of:
/// ```ignore
/// let z = Option::Some::<u32>(10);
/// ```
/// Or the enum can have the type parameters in the `Prefix` variant, case of:
/// ```ignore
/// let z = Option::<u32>::Some(10);
/// ```
/// So we can have type parameters in the `Prefix` or `Regular` variant but not
/// in both.
#[derive(Debug, Clone)]
pub enum TypeArgs {
    /// `Regular` variant indicates the type arguments are located after the suffix.
    Regular(Vec<TypeArgument>),
    /// `Prefix` variant indicates the type arguments are located between the last
    /// prefix and the suffix.
    Prefix(Vec<TypeArgument>),
}

impl TypeArgs {
    pub fn to_vec(&self) -> Vec<TypeArgument> {
        match self {
            TypeArgs::Regular(vec) => vec.to_vec(),
            TypeArgs::Prefix(vec) => vec.to_vec(),
        }
    }

    pub(crate) fn to_vec_mut(&mut self) -> &mut Vec<TypeArgument> {
        match self {
            TypeArgs::Regular(vec) => vec,
            TypeArgs::Prefix(vec) => vec,
        }
    }
}

impl Spanned for TypeArgs {
    fn span(&self) -> Span {
        Span::join_all(self.to_vec().iter().map(|t| t.span()))
    }
}

impl PartialEqWithEngines for TypeArgs {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        match (self, other) {
            (TypeArgs::Regular(vec1), TypeArgs::Regular(vec2)) => vec1.eq(vec2, engines),
            (TypeArgs::Prefix(vec1), TypeArgs::Prefix(vec2)) => vec1.eq(vec2, engines),
            _ => false,
        }
    }
}

impl<T> Spanned for TypeBinding<T> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl PartialEqWithEngines for TypeBinding<()> {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.span == other.span && self.type_arguments.eq(&other.type_arguments, engines)
    }
}

impl<T> TypeBinding<T> {
    pub fn strip_inner(self) -> TypeBinding<()> {
        TypeBinding {
            inner: (),
            type_arguments: self.type_arguments,
            span: self.span,
        }
    }
}

impl TypeBinding<CallPath<(TypeInfo, Ident)>> {
    pub(crate) fn type_check_with_type_info(
        &self,
        ctx: &mut TypeCheckContext,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;

        let (type_info, type_ident) = self.inner.suffix.clone();
        let type_info_span = type_ident.span();

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
            type_info.apply_type_arguments(self.type_arguments.to_vec(), &type_info_span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // resolve the type of the type info object
        let type_id = check!(
            ctx.resolve_type(
                type_engine.insert(decl_engine, type_info),
                &type_info_span,
                EnforceTypeArguments::No,
                Some(&type_info_prefix)
            ),
            type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
            warnings,
            errors
        );

        ok(type_id, warnings, errors)
    }
}

impl TypeBinding<CallPath> {
    pub(crate) fn strip_prefixes(&mut self) {
        self.inner.prefixes = vec![];
    }
}

/// Trait that adds a workaround for easy generic returns in Rust:
/// https://blog.jcoglan.com/2019/04/22/generic-returns-in-rust/
pub(crate) trait TypeCheckTypeBinding<T> {
    fn type_check(
        &mut self,
        ctx: TypeCheckContext,
    ) -> CompileResult<(DeclRef<DeclId<T>>, Option<TypeId>)>;
}

impl TypeCheckTypeBinding<ty::TyFunctionDecl> for TypeBinding<CallPath> {
    fn type_check(
        &mut self,
        mut ctx: TypeCheckContext,
    ) -> CompileResult<(DeclRef<DeclId<ty::TyFunctionDecl>>, Option<TypeId>)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();
        // Grab the declaration.
        let unknown_decl = check!(
            ctx.namespace
                .resolve_call_path_with_visibility_check(engines, &self.inner)
                .cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );
        // Check to see if this is a fn declaration.
        let fn_ref = check!(
            unknown_decl.to_fn_ref(),
            return err(warnings, errors),
            warnings,
            errors
        );
        // Get a new copy from the declaration engine.
        let mut new_copy = decl_engine.get_function(fn_ref.id());
        match self.type_arguments {
            // Monomorphize the copy, in place.
            TypeArgs::Regular(_) => {
                check!(
                    ctx.monomorphize(
                        &mut new_copy,
                        self.type_arguments.to_vec_mut(),
                        EnforceTypeArguments::No,
                        &self.span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
            TypeArgs::Prefix(_) => {
                // Resolve the type arguments without monomorphizing.
                for type_argument in self.type_arguments.to_vec_mut().iter_mut() {
                    check!(
                        ctx.resolve_type(
                            type_argument.type_id,
                            &type_argument.span,
                            EnforceTypeArguments::Yes,
                            None
                        ),
                        type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
                        warnings,
                        errors,
                    );
                }
            }
        }
        // Insert the new copy into the declaration engine.
        let new_fn_ref = ctx.decl_engine.insert(new_copy, todo!());
        ok((new_fn_ref, None), warnings, errors)
    }
}

impl TypeCheckTypeBinding<ty::TyStructDecl> for TypeBinding<CallPath> {
    fn type_check(
        &mut self,
        mut ctx: TypeCheckContext,
    ) -> CompileResult<(DeclRef<DeclId<ty::TyStructDecl>>, Option<TypeId>)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();
        // Grab the declaration.
        let unknown_decl = check!(
            ctx.namespace
                .resolve_call_path_with_visibility_check(engines, &self.inner)
                .cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );
        // Check to see if this is a struct declaration.
        let struct_ref = check!(
            unknown_decl.to_struct_ref(engines),
            return err(warnings, errors),
            warnings,
            errors
        );
        // Get a new copy from the declaration engine.
        let mut new_copy = decl_engine.get_struct(struct_ref.id());
        // Monomorphize the copy, in place.
        check!(
            ctx.monomorphize(
                &mut new_copy,
                self.type_arguments.to_vec_mut(),
                EnforceTypeArguments::No,
                &self.span
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        // Insert the new copy into the declaration engine.
        let new_struct_ref = ctx.decl_engine.insert(new_copy, todo!());
        // Take any trait items that apply to the old type and copy them to the new type.
        let type_id = type_engine.insert(decl_engine, TypeInfo::Struct(new_struct_ref.clone()));
        ctx.namespace
            .insert_trait_implementation_for_type(engines, type_id);
        ok((new_struct_ref, Some(type_id)), warnings, errors)
    }
}

impl TypeCheckTypeBinding<ty::TyEnumDecl> for TypeBinding<CallPath> {
    fn type_check(
        &mut self,
        mut ctx: TypeCheckContext,
    ) -> CompileResult<(DeclRef<DeclId<ty::TyEnumDecl>>, Option<TypeId>)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();
        // Grab the declaration.
        let unknown_decl = check!(
            ctx.namespace
                .resolve_call_path_with_visibility_check(engines, &self.inner)
                .cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Get a new copy from the declaration engine.
        let mut new_copy = if let ty::TyDecl::EnumVariantDecl { decl_id, .. } = &unknown_decl {
            decl_engine.get_enum(decl_id)
        } else {
            // Check to see if this is a enum declaration.
            let enum_ref = check!(
                unknown_decl.to_enum_ref(engines),
                return err(warnings, errors),
                warnings,
                errors
            );
            decl_engine.get_enum(enum_ref.id())
        };

        // Monomorphize the copy, in place.
        check!(
            ctx.monomorphize(
                &mut new_copy,
                self.type_arguments.to_vec_mut(),
                EnforceTypeArguments::No,
                &self.span
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        // Insert the new copy into the declaration engine.
        let new_enum_ref = ctx.decl_engine.insert(new_copy, todo!());
        // Take any trait items that apply to the old type and copy them to the new type.
        let type_id = type_engine.insert(decl_engine, TypeInfo::Enum(new_enum_ref.clone()));
        ctx.namespace
            .insert_trait_implementation_for_type(engines, type_id);
        ok((new_enum_ref, Some(type_id)), warnings, errors)
    }
}

impl TypeCheckTypeBinding<ty::TyConstantDecl> for TypeBinding<CallPath> {
    fn type_check(
        &mut self,
        ctx: TypeCheckContext,
    ) -> CompileResult<(DeclRef<DeclId<ty::TyConstantDecl>>, Option<TypeId>)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let engines = ctx.engines();

        // Grab the declaration.
        let unknown_decl = check!(
            ctx.namespace
                .resolve_call_path_with_visibility_check(engines, &self.inner)
                .cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Check to see if this is a const declaration.
        let const_ref = check!(
            unknown_decl.to_const_ref(),
            return err(warnings, errors),
            warnings,
            errors
        );

        ok((const_ref, None), warnings, errors)
    }
}
