use sway_ast::Intrinsic;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Span, Spanned};

use crate::{
    decl_engine::{DeclEngineInsert, DeclId, DeclRef},
    engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    language::{ty, CallPath, QualifiedCallPath},
    semantic_analysis::{type_check_context::EnforceTypeArguments, TypeCheckContext},
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

    pub fn as_slice(&self) -> &[TypeArgument] {
        match self {
            TypeArgs::Regular(vec) | TypeArgs::Prefix(vec) => vec,
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
        Span::join_all(self.to_vec().iter().map(sway_types::Spanned::span))
    }
}

impl PartialEqWithEngines for TypeArgs {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (TypeArgs::Regular(vec1), TypeArgs::Regular(vec2)) => vec1.eq(vec2, ctx),
            (TypeArgs::Prefix(vec1), TypeArgs::Prefix(vec2)) => vec1.eq(vec2, ctx),
            _ => false,
        }
    }
}

impl<T> Spanned for TypeBinding<T> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl PartialEqWithEngines for TypeBinding<Ident> {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.inner == other.inner
            && self.span == other.span
            && self.type_arguments.eq(&other.type_arguments, ctx)
    }
}

impl PartialEqWithEngines for TypeBinding<Intrinsic> {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.inner == other.inner
            && self.span == other.span
            && self.type_arguments.eq(&other.type_arguments, ctx)
    }
}

impl<T> EqWithEngines for TypeBinding<T> where T: EqWithEngines {}
impl<T> PartialEqWithEngines for TypeBinding<T>
where
    T: PartialEqWithEngines,
{
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.inner.eq(&other.inner, ctx)
            && self.span == other.span
            && self.type_arguments.eq(&other.type_arguments, ctx)
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
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<TypeId, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let (type_info, type_ident) = self.inner.suffix.clone();
        let type_info_span = type_ident.span();

        // find the module that the symbol is in
        let type_info_prefix = ctx.namespace().prepend_module_path(&self.inner.prefixes);
        ctx.namespace()
            .lookup_submodule_from_absolute_path(handler, engines, &type_info_prefix)?;

        // create the type info object
        let type_info = type_info.apply_type_arguments(
            handler,
            self.type_arguments.to_vec(),
            &type_info_span,
        )?;

        // resolve the type of the type info object
        let type_id = ctx
            .resolve_type(
                handler,
                type_engine.insert(engines, type_info, type_info_span.source_id()),
                &type_info_span,
                EnforceTypeArguments::No,
                Some(&type_info_prefix),
            )
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));

        Ok(type_id)
    }
}

impl EqWithEngines for (TypeInfo, Ident) {}
impl PartialEqWithEngines for (TypeInfo, Ident) {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.0.eq(&other.0, ctx) && self.1 == other.1
    }
}

impl TypeBinding<CallPath> {
    pub(crate) fn strip_prefixes(&mut self) {
        self.inner.prefixes = vec![];
    }
}

/// Trait that adds a workaround for easy generic returns in Rust:
/// https://blog.jcoglan.com/2019/04/22/generic-returns-in-rust/
#[allow(clippy::type_complexity)]
pub(crate) trait TypeCheckTypeBinding<T> {
    fn type_check(
        &mut self,
        handler: &Handler,
        ctx: TypeCheckContext,
    ) -> Result<(DeclRef<DeclId<T>>, Option<TypeId>, Option<ty::TyDecl>), ErrorEmitted>;
}

impl TypeCheckTypeBinding<ty::TyFunctionDecl> for TypeBinding<CallPath> {
    fn type_check(
        &mut self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<
        (
            DeclRef<DeclId<ty::TyFunctionDecl>>,
            Option<TypeId>,
            Option<ty::TyDecl>,
        ),
        ErrorEmitted,
    > {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();
        // Grab the declaration.
        let unknown_decl = ctx.resolve_call_path_with_visibility_check(handler, &self.inner)?;
        // Check to see if this is a fn declaration.
        let fn_ref = unknown_decl.to_fn_ref(handler, ctx.engines())?;
        // Get a new copy from the declaration engine.
        let mut new_copy = (*decl_engine.get_function(fn_ref.id())).clone();
        match self.type_arguments {
            // Monomorphize the copy, in place.
            TypeArgs::Regular(_) => {
                ctx.monomorphize(
                    handler,
                    &mut new_copy,
                    self.type_arguments.to_vec_mut(),
                    EnforceTypeArguments::No,
                    &self.span,
                )?;
            }
            TypeArgs::Prefix(_) => {
                // Resolve the type arguments without monomorphizing.
                for type_argument in self.type_arguments.to_vec_mut().iter_mut() {
                    ctx.resolve_type(
                        handler,
                        type_argument.type_id,
                        &type_argument.span,
                        EnforceTypeArguments::Yes,
                        None,
                    )
                    .unwrap_or_else(|err| {
                        type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None)
                    });
                }
            }
        }
        // Insert the new copy into the declaration engine.
        let new_fn_ref = ctx
            .engines
            .de()
            .insert(new_copy)
            .with_parent(ctx.engines.de(), fn_ref.id().into());
        Ok((new_fn_ref, None, None))
    }
}

impl TypeCheckTypeBinding<ty::TyStructDecl> for TypeBinding<CallPath> {
    fn type_check(
        &mut self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<
        (
            DeclRef<DeclId<ty::TyStructDecl>>,
            Option<TypeId>,
            Option<ty::TyDecl>,
        ),
        ErrorEmitted,
    > {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();
        // Grab the declaration.
        let unknown_decl = ctx.resolve_call_path_with_visibility_check(handler, &self.inner)?;
        // Check to see if this is a struct declaration.
        let struct_ref = unknown_decl.to_struct_ref(handler, engines)?;
        // Get a new copy from the declaration engine.
        let mut new_copy = (*decl_engine.get_struct(struct_ref.id())).clone();
        // Monomorphize the copy, in place.
        ctx.monomorphize(
            handler,
            &mut new_copy,
            self.type_arguments.to_vec_mut(),
            EnforceTypeArguments::No,
            &self.span,
        )?;
        // Insert the new copy into the declaration engine.
        let new_struct_ref = ctx.engines.de().insert(new_copy);
        let type_id = type_engine.insert(
            engines,
            TypeInfo::Struct(new_struct_ref.clone()),
            new_struct_ref.span().source_id(),
        );
        Ok((new_struct_ref, Some(type_id), None))
    }
}

impl TypeCheckTypeBinding<ty::TyEnumDecl> for TypeBinding<CallPath> {
    fn type_check(
        &mut self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<
        (
            DeclRef<DeclId<ty::TyEnumDecl>>,
            Option<TypeId>,
            Option<ty::TyDecl>,
        ),
        ErrorEmitted,
    > {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();
        // Grab the declaration.
        let unknown_decl = ctx.resolve_call_path_with_visibility_check(handler, &self.inner)?;

        // Get a new copy from the declaration engine.
        let mut new_copy = if let ty::TyDecl::EnumVariantDecl(ty::EnumVariantDecl {
            enum_ref,
            ..
        }) = &unknown_decl
        {
            (*decl_engine.get_enum(enum_ref.id())).clone()
        } else {
            // Check to see if this is a enum declaration.
            let enum_ref = unknown_decl.to_enum_ref(handler, engines)?;
            (*decl_engine.get_enum(enum_ref.id())).clone()
        };

        // Monomorphize the copy, in place.
        ctx.monomorphize(
            handler,
            &mut new_copy,
            self.type_arguments.to_vec_mut(),
            EnforceTypeArguments::No,
            &self.span,
        )?;
        // Insert the new copy into the declaration engine.
        let new_enum_ref = ctx.engines.de().insert(new_copy);
        let type_id = type_engine.insert(
            engines,
            TypeInfo::Enum(new_enum_ref.clone()),
            new_enum_ref.span().source_id(),
        );
        Ok((new_enum_ref, Some(type_id), Some(unknown_decl)))
    }
}

impl TypeBinding<QualifiedCallPath> {
    pub(crate) fn type_check_qualified(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<DeclRef<DeclId<ty::TyConstantDecl>>, ErrorEmitted> {
        // Grab the declaration.
        let unknown_decl =
            ctx.resolve_qualified_call_path_with_visibility_check(handler, &self.inner)?;

        // Check to see if this is a const declaration.
        let const_ref = unknown_decl.to_const_ref(handler, ctx.engines())?;

        Ok(const_ref)
    }
}

impl TypeCheckTypeBinding<ty::TyConstantDecl> for TypeBinding<CallPath> {
    fn type_check(
        &mut self,
        handler: &Handler,
        ctx: TypeCheckContext,
    ) -> Result<
        (
            DeclRef<DeclId<ty::TyConstantDecl>>,
            Option<TypeId>,
            Option<ty::TyDecl>,
        ),
        ErrorEmitted,
    > {
        // Grab the declaration.
        let unknown_decl = ctx.resolve_call_path_with_visibility_check(handler, &self.inner)?;

        // Check to see if this is a const declaration.
        let const_ref = unknown_decl.to_const_ref(handler, ctx.engines())?;

        Ok((const_ref, None, None))
    }
}
