use sway_ast::Intrinsic;
use sway_error::{
    convert_parse_tree_error::ConvertParseTreeError,
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Span, Spanned};

use crate::{
    decl_engine::{
        parsed_id::ParsedDeclId, DeclEngineGetParsedDeclId, DeclEngineInsert, DeclId, DeclRef,
    },
    engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    language::{
        parsed::{
            ConstantDeclaration, Declaration, EnumDeclaration, EnumVariantDeclaration,
            FunctionDeclaration, StructDeclaration,
        },
        ty, CallPath, QualifiedCallPath,
    },
    semantic_analysis::{symbol_resolve_context::SymbolResolveContext, TypeCheckContext},
    transform::to_parsed_lang::type_name_to_type_info_opt,
    type_system::priv_prelude::*,
    EnforceTypeArguments, Ident,
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

#[allow(clippy::type_complexity)]
pub trait SymbolResolveTypeBinding<T> {
    fn resolve_symbol(
        &mut self,
        handler: &Handler,
        ctx: SymbolResolveContext,
    ) -> Result<T, ErrorEmitted>;
}

impl SymbolResolveTypeBinding<ParsedDeclId<FunctionDeclaration>> for TypeBinding<CallPath> {
    fn resolve_symbol(
        &mut self,
        handler: &Handler,
        ctx: SymbolResolveContext,
    ) -> Result<ParsedDeclId<FunctionDeclaration>, ErrorEmitted> {
        let engines = ctx.engines();
        // Grab the declaration.
        let unknown_decl = ctx.resolve_call_path_with_visibility_check(handler, &self.inner)?;
        // Check to see if this is a function declaration.
        let fn_decl = unknown_decl
            .resolve_parsed(engines.de())
            .to_fn_decl(handler, engines)?;
        Ok(fn_decl)
    }
}

impl SymbolResolveTypeBinding<ParsedDeclId<EnumDeclaration>> for TypeBinding<CallPath> {
    fn resolve_symbol(
        &mut self,
        handler: &Handler,
        ctx: SymbolResolveContext,
    ) -> Result<ParsedDeclId<EnumDeclaration>, ErrorEmitted> {
        let engines = ctx.engines();

        // Grab the declaration.
        let unknown_decl = ctx
            .resolve_call_path_with_visibility_check(handler, &self.inner)?
            .expect_parsed();

        // Check to see if this is a enum declaration.
        let enum_decl = if let Declaration::EnumVariantDeclaration(EnumVariantDeclaration {
            enum_ref,
            ..
        }) = &unknown_decl
        {
            *enum_ref
        } else {
            // Check to see if this is a enum declaration.
            unknown_decl.to_enum_decl(handler, engines)?
        };

        Ok(enum_decl)
    }
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
        let unknown_decl = ctx
            .resolve_call_path_with_visibility_check(handler, &self.inner)?
            .expect_typed();
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
        let new_fn_ref = decl_engine
            .insert(
                new_copy,
                decl_engine.get_parsed_decl_id(fn_ref.id()).as_ref(),
            )
            .with_parent(ctx.engines.de(), fn_ref.id().into());
        Ok((new_fn_ref, None, None))
    }
}

impl SymbolResolveTypeBinding<ParsedDeclId<StructDeclaration>> for TypeBinding<CallPath> {
    fn resolve_symbol(
        &mut self,
        handler: &Handler,
        ctx: SymbolResolveContext,
    ) -> Result<ParsedDeclId<StructDeclaration>, ErrorEmitted> {
        let engines = ctx.engines();
        // Grab the declaration.
        let unknown_decl = ctx.resolve_call_path_with_visibility_check(handler, &self.inner)?;

        // Check to see if this is a struct declaration.
        let struct_decl = unknown_decl.to_struct_decl(handler, engines)?;
        struct_decl
            .resolve_parsed(engines.de())
            .to_struct_decl(handler, engines)
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
        let unknown_decl = ctx
            .resolve_call_path_with_visibility_check(handler, &self.inner)?
            .expect_typed();
        // Check to see if this is a struct declaration.
        let struct_id = unknown_decl.to_struct_decl(handler, engines)?;
        // Get a new copy from the declaration engine.
        let mut new_copy = (*decl_engine.get_struct(&struct_id)).clone();
        // Monomorphize the copy, in place.
        ctx.monomorphize(
            handler,
            &mut new_copy,
            self.type_arguments.to_vec_mut(),
            EnforceTypeArguments::No,
            &self.span,
        )?;
        // Insert the new copy into the declaration engine.
        let new_struct_ref = decl_engine.insert(
            new_copy,
            decl_engine.get_parsed_decl_id(&struct_id).as_ref(),
        );
        let type_id = type_engine.insert(
            engines,
            TypeInfo::Struct(*new_struct_ref.id()),
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
        let unknown_decl = ctx
            .resolve_call_path_with_visibility_check(handler, &self.inner)?
            .expect_typed();

        // Get a new copy from the declaration engine.
        let enum_id = if let ty::TyDecl::EnumVariantDecl(ty::EnumVariantDecl { enum_ref, .. }) =
            &unknown_decl
        {
            *enum_ref.id()
        } else {
            // Check to see if this is a enum declaration.
            unknown_decl.to_enum_id(handler, engines)?
        };

        let mut new_copy = (*decl_engine.get_enum(&enum_id)).clone();

        // Monomorphize the copy, in place.
        ctx.monomorphize(
            handler,
            &mut new_copy,
            self.type_arguments.to_vec_mut(),
            EnforceTypeArguments::No,
            &self.span,
        )?;
        // Insert the new copy into the declaration engine.
        let new_enum_ref =
            decl_engine.insert(new_copy, decl_engine.get_parsed_decl_id(&enum_id).as_ref());
        let type_id = type_engine.insert(
            engines,
            TypeInfo::Enum(*new_enum_ref.id()),
            new_enum_ref.span().source_id(),
        );
        Ok((new_enum_ref, Some(type_id), Some(unknown_decl)))
    }
}

impl TypeBinding<QualifiedCallPath> {
    pub fn resolve_symbol(
        &mut self,
        handler: &Handler,
        mut ctx: SymbolResolveContext,
        span: Span,
    ) -> Result<Declaration, ErrorEmitted> {
        let engines = ctx.engines();

        println!("resolve_symbol {:?}", self.inner);

        // The first step is to determine if the call path refers to a module,
        // enum, function or constant.
        // If only one exists, then we use that one. Otherwise, if more than one exist, it is
        // an ambiguous reference error.

        let mut is_module = false;
        let mut maybe_function: Option<(ParsedDeclId<FunctionDeclaration>, _)> = None;
        let mut maybe_enum: Option<(ParsedDeclId<EnumDeclaration>, _, _)> = None;

        let module_probe_handler = Handler::default();
        let function_probe_handler = Handler::default();
        let enum_probe_handler = Handler::default();
        let const_probe_handler = Handler::default();

        if self.inner.qualified_path_root.is_none() {
            // Check if this could be a module
            is_module = {
                let call_path_binding = self.clone();
                ctx.namespace().program_id(engines).read(engines, |m| {
                    m.lookup_submodule(
                        &module_probe_handler,
                        engines,
                        &[
                            call_path_binding.inner.call_path.prefixes.clone(),
                            vec![call_path_binding.inner.call_path.suffix.clone()],
                        ]
                        .concat(),
                    )
                    .ok()
                    .is_some()
                })
            };

            // Check if this could be a function
            maybe_function = {
                let call_path_binding = self.clone();
                let mut call_path_binding = TypeBinding {
                    inner: call_path_binding.inner.call_path,
                    type_arguments: call_path_binding.type_arguments,
                    span: call_path_binding.span,
                };

                let result: Result<ParsedDeclId<FunctionDeclaration>, ErrorEmitted> =
                    SymbolResolveTypeBinding::resolve_symbol(
                        &mut call_path_binding,
                        &function_probe_handler,
                        ctx.by_ref(),
                    );

                result.ok().map(|fn_ref| (fn_ref, call_path_binding))
            };

            // Check if this could be an enum
            maybe_enum = {
                let call_path_binding = self.clone();
                let variant_name = call_path_binding.inner.call_path.suffix.clone();
                let enum_call_path = call_path_binding.inner.call_path.rshift();

                let mut call_path_binding = TypeBinding {
                    inner: enum_call_path,
                    type_arguments: call_path_binding.type_arguments,
                    span: call_path_binding.span,
                };

                let result: Result<ParsedDeclId<EnumDeclaration>, ErrorEmitted> =
                    SymbolResolveTypeBinding::resolve_symbol(
                        &mut call_path_binding,
                        &enum_probe_handler,
                        ctx.by_ref(),
                    );

                result
                    .ok()
                    .map(|enum_ref| (enum_ref, variant_name, call_path_binding))
            };
        };

        // Check if this could be a constant
        let maybe_const = SymbolResolveTypeBinding::<(
            ParsedDeclId<ConstantDeclaration>,
            TypeBinding<CallPath>,
        )>::resolve_symbol(self, &const_probe_handler, ctx.by_ref())
        .ok();

        // compare the results of the checks
        let exp = match (is_module, maybe_function, maybe_enum, maybe_const) {
            (false, None, Some((enum_ref, _variant_name, _call_path_binding)), None) => {
                handler.append(enum_probe_handler);
                Declaration::EnumDeclaration(enum_ref)
            }
            (false, Some((fn_ref, call_path_binding)), None, None) => {
                handler.append(function_probe_handler);
                // In case `foo::bar::<TyArgs>::baz(...)` throw an error.
                if let TypeArgs::Prefix(_) = call_path_binding.type_arguments {
                    handler.emit_err(
                        ConvertParseTreeError::GenericsNotSupportedHere {
                            span: call_path_binding.type_arguments.span(),
                        }
                        .into(),
                    );
                }
                Declaration::FunctionDeclaration(fn_ref)
            }
            (true, None, None, None) => {
                handler.append(module_probe_handler);
                return Err(handler.emit_err(CompileError::ModulePathIsNotAnExpression {
                    module_path: self.inner.call_path.to_string(),
                    span,
                }));
            }
            (false, None, None, Some((const_ref, call_path_binding))) => {
                handler.append(const_probe_handler);
                if !call_path_binding.type_arguments.to_vec().is_empty() {
                    // In case `foo::bar::CONST::<TyArgs>` throw an error.
                    // In case `foo::bar::<TyArgs>::CONST` throw an error.
                    handler.emit_err(
                        ConvertParseTreeError::GenericsNotSupportedHere {
                            span: self.type_arguments.span(),
                        }
                        .into(),
                    );
                }
                Declaration::ConstantDeclaration(const_ref)
            }
            (false, None, None, None) => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: self.inner.call_path.suffix.clone(),
                    span: self.inner.call_path.suffix.span(),
                }));
            }
            _ => {
                return Err(handler.emit_err(CompileError::AmbiguousPath { span }));
            }
        };

        Ok(exp)
    }
}

impl SymbolResolveTypeBinding<ParsedDeclId<ConstantDeclaration>> for TypeBinding<CallPath> {
    fn resolve_symbol(
        &mut self,
        handler: &Handler,
        ctx: SymbolResolveContext,
    ) -> Result<ParsedDeclId<ConstantDeclaration>, ErrorEmitted> {
        println!(
            "resolve_symbol SymbolResolveTypeBinding Constant {:?}",
            self
        );

        // Grab the declaration.
        let unknown_decl = ctx
            .resolve_call_path_with_visibility_check(handler, &self.inner)?
            .expect_parsed();

        // Check to see if this is a const declaration.
        let const_ref = unknown_decl.to_const_decl(handler, ctx.engines())?;

        Ok(const_ref)
    }
}

impl SymbolResolveTypeBinding<(ParsedDeclId<ConstantDeclaration>, TypeBinding<CallPath>)>
    for TypeBinding<QualifiedCallPath>
{
    fn resolve_symbol(
        &mut self,
        handler: &Handler,
        mut ctx: SymbolResolveContext,
    ) -> Result<(ParsedDeclId<ConstantDeclaration>, TypeBinding<CallPath>), ErrorEmitted> {
        let mut call_path_binding = TypeBinding {
            inner: self.inner.call_path.clone(),
            type_arguments: self.type_arguments.clone(),
            span: self.span.clone(),
        };

        println!(
            "resolve_symbol SymbolResolveTypeBinding Constant {:?}",
            call_path_binding
        );

        let type_info_opt = call_path_binding
            .clone()
            .inner
            .prefixes
            .last()
            .map(|type_name| {
                type_name_to_type_info_opt(type_name).unwrap_or(TypeInfo::Custom {
                    qualified_call_path: type_name.clone().into(),
                    type_arguments: None,
                    root_type_id: None,
                })
            });

        if let Some(type_info) = type_info_opt {
            if TypeInfo::is_self_type(&type_info) {
                call_path_binding.strip_prefixes();
            }
        }

        let const_res: Result<ParsedDeclId<ConstantDeclaration>, ErrorEmitted> =
            SymbolResolveTypeBinding::resolve_symbol(
                &mut call_path_binding,
                &Handler::default(),
                ctx.by_ref(),
            );
        if const_res.is_ok() {
            return const_res.map(|const_ref| (const_ref, call_path_binding.clone()));
        }

        // If we didn't find a constant, check for the constant inside the impl.
        let unknown_decl = ctx
            .resolve_qualified_call_path_with_visibility_check(handler, &self.inner)?
            .expect_parsed();

        // Check to see if this is a const declaration.
        let const_ref = unknown_decl.to_const_decl(handler, ctx.engines())?;

        Ok((const_ref, call_path_binding.clone()))
    }
}

impl TypeBinding<QualifiedCallPath> {
    pub(crate) fn type_check_qualified(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<DeclRef<DeclId<ty::TyConstantDecl>>, ErrorEmitted> {
        // Grab the declaration.
        let unknown_decl = ctx
            .resolve_qualified_call_path_with_visibility_check(handler, &self.inner)?
            .expect_typed();

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
        let unknown_decl = ctx
            .resolve_call_path_with_visibility_check(handler, &self.inner)?
            .expect_typed();

        // Check to see if this is a const declaration.
        let const_ref = unknown_decl.to_const_ref(handler, ctx.engines())?;

        Ok((const_ref, None, None))
    }
}
