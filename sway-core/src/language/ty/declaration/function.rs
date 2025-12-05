use crate::{
    ast_elements::type_argument::GenericTypeArgument,
    decl_engine::*,
    engine_threading::*,
    has_changes,
    language::{
        parsed::{self, FunctionDeclaration, FunctionDeclarationKind},
        ty::*,
        CallPath, Inline, Purity, Trace, Visibility,
    },
    semantic_analysis::TypeCheckContext,
    transform::{self, AttributeKind},
    type_system::*,
    types::*,
};
use ast_elements::type_parameter::ConstGenericExpr;
use either::Either;
use monomorphization::MonomorphizeHelper;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt,
    hash::{Hash, Hasher},
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_macros::Visit;
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TyFunctionDeclKind {
    Default,
    Entry,
    Main,
    Test,
}

#[derive(Clone, Debug, Serialize, Deserialize, Visit)]
pub struct TyFunctionDecl {
    #[visit(skip)]
    pub name: Ident,
    pub body: TyCodeBlock,
    pub parameters: Vec<TyFunctionParameter>,
    /// The [TyDecl] in which this function is implemented.
    ///
    /// For [TyFunctionDecl]s representing _declarations_ of
    /// trait or ABI provided functions and methods, this will be
    /// the [TyDecl::TraitDecl] and [TyDecl::AbiDecl], respectively.
    ///
    /// For [TyFunctionDecl]s representing _implementations_ of
    /// functions and methods in trait or self impls, this will be
    /// the [TyDecl::ImplSelfOrTrait].
    ///
    /// **For [TyFunctionDecl]s representing _function applications_,
    /// this will always be the [TyDecl::ImplSelfOrTrait], even if
    /// the called function is a trait or ABI provided function.**
    ///
    /// `None` for module functions.
    pub implementing_type: Option<TyDecl>,
    /// The [TypeId] of the type that this function is implemented for.
    ///
    /// For [TyFunctionDecl]s representing _declarations_ of
    /// trait or ABI provided functions and methods, this will be
    /// the [TypeInfo::UnknownGeneric] representing the `Self` generic parameter.
    ///
    /// For [TyFunctionDecl]s representing _implementations_ of
    /// functions and methods in trait or self impls, this will be
    /// the [TypeInfo] of the corresponding `Self` type, e.g., [TypeInfo::Struct].
    ///
    /// **For [TyFunctionDecl]s representing _function applications_,
    /// this will always be the [TypeInfo] of the corresponding `Self` type,
    /// even if the called function is a trait or ABI provided function.**
    ///
    /// `None` for module functions.
    pub implementing_for: Option<TypeId>,
    #[visit(skip)]
    pub span: Span,
    /// For module functions, this is the full call path of the function.
    ///
    /// Otherwise, the [CallPath::prefixes] are the prefixes of the module
    /// in which the defining [TyFunctionDecl] is located, and the
    /// [CallPath::suffix] is the function name.
    #[visit(skip)]
    pub call_path: CallPath,
    #[visit(skip)]
    pub attributes: transform::Attributes,
    pub type_parameters: Vec<TypeParameter>,
    pub return_type: GenericTypeArgument,
    #[visit(skip)]
    pub visibility: Visibility,
    /// Whether this function exists in another contract and requires a call to it or not.
    #[visit(skip)]
    pub is_contract_call: bool,
    #[visit(skip)]
    pub purity: Purity,
    #[visit(skip)]
    pub where_clause: Vec<(Ident, Vec<TraitConstraint>)>,
    #[visit(skip)]
    pub is_trait_method_dummy: bool,
    #[visit(skip)]
    pub is_type_check_finalized: bool,
    /// !!! WARNING !!!
    /// This field is currently not reliable.
    /// Do not use it to check the function kind.
    /// Instead, use the [Self::is_default], [Self::is_entry], [Self::is_main], and [Self::is_test] methods.
    /// TODO: See: https://github.com/FuelLabs/sway/issues/7371
    /// !!! WARNING !!!
    #[visit(skip)]
    pub kind: TyFunctionDeclKind,
}

impl TyDeclParsedType for TyFunctionDecl {
    type ParsedType = FunctionDeclaration;
}

impl DebugWithEngines for TyFunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "{}{:?}{}({}): {:?} -> {:?}",
            if self.is_trait_method_dummy {
                "dummy ".to_string()
            } else {
                "".to_string()
            },
            self.name,
            if !self.type_parameters.is_empty() {
                format!(
                    "<{}>",
                    self.type_parameters
                        .iter()
                        .map(|p| format!("{:?}", engines.help_out(p)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                "".to_string()
            },
            self.parameters
                .iter()
                .map(|p| format!(
                    "{}: {:?} -> {:?}",
                    p.name.as_str(),
                    engines.help_out(p.type_argument.initial_type_id),
                    engines.help_out(p.type_argument.type_id)
                ))
                .collect::<Vec<_>>()
                .join(", "),
            engines.help_out(self.return_type.initial_type_id),
            engines.help_out(self.return_type.type_id),
        )
    }
}

impl DisplayWithEngines for TyFunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "{}{}({}) -> {}",
            self.name,
            if !self.type_parameters.is_empty() {
                format!(
                    "<{}>",
                    self.type_parameters
                        .iter()
                        .map(|p| {
                            let p = p
                                .as_type_parameter()
                                .expect("only works for type parameters");
                            format!("{}", engines.help_out(p.initial_type_id))
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                "".to_string()
            },
            self.parameters
                .iter()
                .map(|p| format!(
                    "{}: {}",
                    p.name.as_str(),
                    engines.help_out(p.type_argument.initial_type_id)
                ))
                .collect::<Vec<_>>()
                .join(", "),
            engines.help_out(self.return_type.initial_type_id),
        )
    }
}

impl MaterializeConstGenerics for TyFunctionDecl {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        for tp in self.type_parameters.iter_mut() {
            match tp {
                TypeParameter::Type(p) => p
                    .type_id
                    .materialize_const_generics(engines, handler, name, value)?,
                TypeParameter::Const(p) if p.name.as_str() == name => match p.expr.as_ref() {
                    Some(v) => {
                        assert!(
                            v.as_literal_val().unwrap() as u64
                                == value
                                    .extract_literal_value()
                                    .unwrap()
                                    .cast_value_to_u64()
                                    .unwrap()
                        );
                    }
                    None => {
                        p.expr = Some(ConstGenericExpr::from_ty_expression(handler, value)?);
                    }
                },
                _ => {}
            }
        }

        for param in self.parameters.iter_mut() {
            param
                .type_argument
                .type_id
                .materialize_const_generics(engines, handler, name, value)?;
        }
        self.return_type
            .type_id
            .materialize_const_generics(engines, handler, name, value)?;
        self.body
            .materialize_const_generics(engines, handler, name, value)
    }
}

/// Rename const generics when the name inside the struct/enum declaration  does not match
/// the name in the impl.
fn rename_const_generics_on_function(
    engines: &Engines,
    impl_self_or_trait: &TyImplSelfOrTrait,
    function: &mut TyFunctionDecl,
) {
    let from = impl_self_or_trait.implementing_for.initial_type_id;
    let to = impl_self_or_trait.implementing_for.type_id;

    let from = engines.te().get(from);
    let to = engines.te().get(to);

    match (&*from, &*to) {
        (
            TypeInfo::Custom {
                type_arguments: Some(type_arguments),
                ..
            },
            TypeInfo::Struct(s),
        ) => {
            let decl = engines.de().get(s);
            rename_const_generics_on_function_inner(
                engines,
                function,
                type_arguments,
                decl.type_parameters(),
            );
        }
        (
            TypeInfo::Custom {
                type_arguments: Some(type_arguments),
                ..
            },
            TypeInfo::Enum(s),
        ) => {
            let decl = engines.de().get(s);
            rename_const_generics_on_function_inner(
                engines,
                function,
                type_arguments,
                decl.type_parameters(),
            );
        }
        _ => (),
    }
}

fn rename_const_generics_on_function_inner(
    engines: &Engines,
    function: &mut TyFunctionDecl,
    type_arguments: &[GenericArgument],
    generic_parameters: &[TypeParameter],
) {
    for a in type_arguments.iter().zip(generic_parameters.iter()) {
        match (a.0, a.1) {
            (GenericArgument::Type(a), TypeParameter::Const(b)) => {
                // replace all references from "a.name.as_str()" to "b.name.as_str()"
                let mut type_subst_map = TypeSubstMap::default();
                type_subst_map.const_generics_renaming.insert(
                    a.call_path_tree
                        .as_ref()
                        .unwrap()
                        .qualified_call_path
                        .call_path
                        .suffix
                        .clone(),
                    b.name.clone(),
                );
                function.subst_inner(&SubstTypesContext {
                    engines,
                    type_subst_map: Some(&type_subst_map),
                    subst_function_body: true,
                });
            }
            (GenericArgument::Const(a), TypeParameter::Const(b)) => {
                engines
                    .obs()
                    .trace(|| format!("{:?} -> {:?}", a.expr, b.expr));
            }
            _ => {}
        }
    }
}

impl DeclRefFunction {
    /// Makes method with a copy of type_id.
    /// This avoids altering the type_id already in the type map.
    /// Without this it is possible to retrieve a method from the type map unify its types and
    /// the second time it won't be possible to retrieve the same method.
    pub fn get_method_safe_to_unify(&self, engines: &Engines, type_id: TypeId) -> Self {
        engines.obs().trace(|| {
            format!(
                "    before get_method_safe_to_unify: {:?} {:?}",
                engines.help_out(type_id),
                engines.help_out(self.id())
            )
        });

        let decl_engine = engines.de();

        let original = &*decl_engine.get_function(self);
        let mut method = original.clone();

        if let Some(method_implementing_for) = method.implementing_for {
            let mut type_id_type_subst_map = TypeSubstMap::new();

            if let Some(TyDecl::ImplSelfOrTrait(t)) = method.implementing_type.clone() {
                let impl_self_or_trait = &*engines.de().get(&t.decl_id);
                rename_const_generics_on_function(engines, impl_self_or_trait, &mut method);

                let mut type_id_type_parameters = vec![];
                let mut const_generic_parameters = BTreeMap::default();
                type_id.extract_type_parameters(
                    engines,
                    0,
                    &mut type_id_type_parameters,
                    &mut const_generic_parameters,
                    impl_self_or_trait.implementing_for.type_id,
                );

                type_id_type_subst_map
                    .const_generics_materialization
                    .append(&mut const_generic_parameters);

                for p in impl_self_or_trait
                    .impl_type_parameters
                    .iter()
                    .filter_map(|x| x.as_type_parameter())
                {
                    let matches = type_id_type_parameters
                        .iter()
                        .filter(|(_, orig_tp)| {
                            engines.te().get(*orig_tp).eq(
                                &*engines.te().get(p.type_id),
                                &PartialEqWithEnginesContext::new(engines),
                            )
                        })
                        .collect::<Vec<_>>();

                    if !matches.is_empty() {
                        // Adds type substitution for first match only as we can apply only one.
                        type_id_type_subst_map.insert(p.type_id, matches[0].0);
                    } else if engines
                        .te()
                        .get(impl_self_or_trait.implementing_for.initial_type_id)
                        .eq(
                            &*engines.te().get(p.initial_type_id),
                            &PartialEqWithEnginesContext::new(engines),
                        )
                    {
                        type_id_type_subst_map.insert(p.type_id, type_id);
                    }
                }
            }

            // Duplicate arguments to avoid changing TypeId inside TraitMap
            for parameter in method.parameters.iter_mut() {
                parameter.type_argument.type_id = engines
                    .te()
                    .duplicate(engines, parameter.type_argument.type_id)
            }

            let mut method_type_subst_map = TypeSubstMap::new();
            method_type_subst_map.extend(&type_id_type_subst_map);
            method_type_subst_map.insert(method_implementing_for, type_id);

            method.subst(&SubstTypesContext::new(
                engines,
                &method_type_subst_map,
                true,
            ));

            let r = engines
                .de()
                .insert(
                    method.clone(),
                    engines.de().get_parsed_decl_id(self.id()).as_ref(),
                )
                .with_parent(decl_engine, self.id().into());

            engines.obs().trace(|| {
                format!(
                    "    after get_method_safe_to_unify: {:?}; {:?}",
                    engines.help_out(type_id),
                    engines.help_out(r.id())
                )
            });

            return r;
        }

        engines.obs().trace(|| {
            format!(
                "    after get_method_safe_to_unify: {:?}; {:?}",
                engines.help_out(type_id),
                engines.help_out(self.id())
            )
        });

        self.clone()
    }
}

impl Named for TyFunctionDecl {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl IsConcrete for TyFunctionDecl {
    fn is_concrete(&self, engines: &Engines) -> bool {
        self.type_parameters
            .iter()
            .all(|tp| tp.is_concrete(engines))
            && self
                .return_type
                .type_id
                .is_concrete(engines, TreatNumericAs::Concrete)
            && self.parameters().iter().all(|t| {
                t.type_argument
                    .type_id
                    .is_concrete(engines, TreatNumericAs::Concrete)
            })
    }
}
impl declaration::FunctionSignature for TyFunctionDecl {
    fn parameters(&self) -> &Vec<TyFunctionParameter> {
        &self.parameters
    }

    fn return_type(&self) -> &GenericTypeArgument {
        &self.return_type
    }
}

impl EqWithEngines for TyFunctionDecl {}
impl PartialEqWithEngines for TyFunctionDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.body.eq(&other.body, ctx)
            && self.parameters.eq(&other.parameters, ctx)
            && self.return_type.eq(&other.return_type, ctx)
            && self.type_parameters.eq(&other.type_parameters, ctx)
            && self.visibility == other.visibility
            && self.is_contract_call == other.is_contract_call
            && self.purity == other.purity
            && self.call_path == other.call_path
            && self.span == other.span
    }
}

impl HashWithEngines for TyFunctionDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyFunctionDecl {
            name,
            body,
            parameters,
            return_type,
            type_parameters,
            visibility,
            is_contract_call,
            purity,
            call_path,
            span,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            implementing_type: _,
            implementing_for: _,
            where_clause: _,
            is_trait_method_dummy: _,
            is_type_check_finalized: _,
            kind: _,
        } = self;
        name.hash(state);
        body.hash(state, engines);
        parameters.hash(state, engines);
        return_type.hash(state, engines);
        type_parameters.hash(state, engines);
        visibility.hash(state);
        is_contract_call.hash(state);
        purity.hash(state);
        call_path.hash(state);
        span.hash(state);
    }
}

impl SubstTypes for TyFunctionDecl {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let changes = if ctx.subst_function_body {
            has_changes! {
                self.type_parameters.subst(ctx);
                self.parameters.subst(ctx);
                self.return_type.subst(ctx);
                self.body.subst(ctx);
                self.implementing_for.subst(ctx);
            }
        } else {
            has_changes! {
                self.type_parameters.subst(ctx);
                self.parameters.subst(ctx);
                self.return_type.subst(ctx);
                self.implementing_for.subst(ctx);
            }
        };

        if let Some(map) = ctx.type_subst_map.as_ref() {
            let handler = Handler::default();
            for (name, value) in &map.const_generics_materialization {
                let _ = self.materialize_const_generics(ctx.engines, &handler, name, value);
            }
            HasChanges::Yes
        } else {
            changes
        }
    }
}

impl ReplaceDecls for TyFunctionDecl {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        let mut func_ctx = ctx.by_ref().with_self_type(self.implementing_for);
        self.body
            .replace_decls(decl_mapping, handler, &mut func_ctx)
    }
}

impl Spanned for TyFunctionDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyFunctionDecl {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }

    fn has_self_type_param(&self) -> bool {
        false
    }

    fn materialize_const_generics2(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        // dbg!(&self);
        let mut cow = std::borrow::Cow::Borrowed(self);
        let mut visitor = MaterializeConstGenericsVisitor {
            engines,
            handler,
            name,
            value,
        };
        TyFunctionDecl::visit(&mut cow, &mut visitor);
        if let std::borrow::Cow::Owned(new_fn) = cow {
            *self = new_fn
        }
        // dbg!(&self);
        Ok(())
    }
}

impl CollectTypesMetadata for TyFunctionDecl {
    fn collect_types_metadata(
        &self,
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        let mut body = vec![];
        for content in self.body.contents.iter() {
            body.append(&mut content.collect_types_metadata(handler, ctx)?);
        }
        body.append(
            &mut self
                .return_type
                .type_id
                .collect_types_metadata(handler, ctx)?,
        );
        for p in self.type_parameters.iter() {
            let p = p
                .as_type_parameter()
                .expect("only works for type parameters");
            body.append(&mut p.type_id.collect_types_metadata(handler, ctx)?);
        }
        for param in self.parameters.iter() {
            body.append(
                &mut param
                    .type_argument
                    .type_id
                    .collect_types_metadata(handler, ctx)?,
            );
        }
        Ok(body)
    }
}

impl TyFunctionDecl {
    pub(crate) fn set_implementing_type(&mut self, decl: TyDecl) {
        self.implementing_type = Some(decl);
    }

    /// Used to create a stubbed out function when the function fails to
    /// compile, preventing cascading namespace errors.
    pub(crate) fn error(decl: &parsed::FunctionDeclaration) -> TyFunctionDecl {
        let parsed::FunctionDeclaration {
            name,
            return_type,
            span,
            visibility,
            purity,
            where_clause,
            kind,
            ..
        } = decl;
        TyFunctionDecl {
            purity: *purity,
            name: name.clone(),
            body: <_>::default(),
            implementing_type: None,
            implementing_for: None,
            span: span.clone(),
            call_path: CallPath::from(Ident::dummy()),
            attributes: Default::default(),
            is_contract_call: false,
            parameters: Default::default(),
            visibility: *visibility,
            return_type: return_type.clone(),
            type_parameters: Default::default(),
            where_clause: where_clause.clone(),
            is_trait_method_dummy: false,
            is_type_check_finalized: true,
            kind: match kind {
                FunctionDeclarationKind::Default => TyFunctionDeclKind::Default,
                FunctionDeclarationKind::Entry => TyFunctionDeclKind::Entry,
                FunctionDeclarationKind::Test => TyFunctionDeclKind::Test,
                FunctionDeclarationKind::Main => TyFunctionDeclKind::Main,
            },
        }
    }

    /// If there are parameters, join their spans. Otherwise, use the fn name span.
    pub(crate) fn parameters_span(&self) -> Span {
        if !self.parameters.is_empty() {
            self.parameters.iter().fold(
                // TODO: Use Span::join_all().
                self.parameters[0].name.span(),
                |acc, TyFunctionParameter { type_argument, .. }| {
                    Span::join(acc, &type_argument.span)
                },
            )
        } else {
            self.name.span()
        }
    }

    pub fn to_fn_selector_value_untruncated(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<Vec<u8>, ErrorEmitted> {
        let mut hasher = Sha256::new();
        let data = self.to_selector_name(handler, engines)?;
        hasher.update(data);
        let hash = hasher.finalize();
        Ok(hash.to_vec())
    }

    /// Converts a [TyFunctionDecl] into a value that is to be used in contract function
    /// selectors.
    /// Hashes the name and parameters using SHA256, and then truncates to four bytes.
    pub fn to_fn_selector_value(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<[u8; 4], ErrorEmitted> {
        let hash = self.to_fn_selector_value_untruncated(handler, engines)?;
        // 4 bytes truncation via copying into a 4 byte buffer
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&hash[..4]);
        Ok(buf)
    }

    pub fn to_selector_name(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<String, ErrorEmitted> {
        let named_params = self
            .parameters
            .iter()
            .map(|TyFunctionParameter { type_argument, .. }| {
                engines
                    .te()
                    .to_typeinfo(type_argument.type_id, &type_argument.span)
                    .expect("unreachable I think?")
                    .to_selector_name(handler, engines, &type_argument.span)
            })
            .filter_map(|name| name.ok())
            .collect::<Vec<String>>();

        Ok(format!(
            "{}({})",
            self.name.as_str(),
            named_params.join(","),
        ))
    }

    pub fn is_default(&self) -> bool {
        // TODO: Properly implement `TyFunctionDecl::kind` and match kind to `Default`.
        //       See: https://github.com/FuelLabs/sway/issues/7371
        !(self.is_entry() || self.is_main() || self.is_test())
    }

    /// Whether or not this function is the default entry point.
    pub fn is_entry(&self) -> bool {
        matches!(self.kind, TyFunctionDeclKind::Entry)
    }

    pub fn is_main(&self) -> bool {
        matches!(self.kind, TyFunctionDeclKind::Main)
    }

    /// Whether or not this function is a unit test, i.e. decorated with `#[test]`.
    pub fn is_test(&self) -> bool {
        // TODO: Properly implement `TyFunctionDecl::kind` and match kind to `Test`.
        //       See: https://github.com/FuelLabs/sway/issues/7371
        self.attributes.has_any_of_kind(AttributeKind::Test)
    }

    pub fn inline(&self) -> Option<Inline> {
        self.attributes.inline()
    }

    pub fn trace(&self) -> Option<Trace> {
        self.attributes.trace()
    }

    pub fn is_fallback(&self) -> bool {
        self.attributes.has_any_of_kind(AttributeKind::Fallback)
    }

    /// Whether or not this function is a constructor for the type given by `type_id`.
    ///
    /// Returns `Some(true)` if the function is surely the constructor and `Some(false)` if
    /// it is surely not a constructor, and `None` if it cannot decide.
    pub fn is_constructor(&self, engines: &Engines, type_id: TypeId) -> Option<bool> {
        if self
            .parameters
            .first()
            .map(|param| param.is_self())
            .unwrap_or_default()
        {
            return Some(false);
        };

        match &self.implementing_type {
            Some(TyDecl::ImplSelfOrTrait(t)) => {
                let unify_check = UnifyCheck::non_dynamic_equality(engines);

                let implementing_for = engines.de().get(&t.decl_id).implementing_for.type_id;

                // TODO: Implement the check in detail for all possible cases (e.g. trait impls for generics etc.)
                //       and return just the definite `bool` and not `Option<bool>`.
                //       That would be too much effort at the moment for the immediate practical need of
                //       error reporting where we suggest obvious most common constructors
                //       that will be found using this simple check.
                if unify_check.check(type_id, implementing_for)
                    && unify_check.check(type_id, self.return_type.type_id)
                {
                    Some(true)
                } else {
                    None
                }
            }
            _ => Some(false),
        }
    }

    pub fn is_from_blanket_impl(&self, engines: &Engines) -> bool {
        if let Some(TyDecl::ImplSelfOrTrait(existing_impl_trait)) = &self.implementing_type {
            let existing_trait_decl = engines
                .de()
                .get_impl_self_or_trait(&existing_impl_trait.decl_id);
            if !existing_trait_decl.impl_type_parameters.is_empty()
                && matches!(
                    *engines
                        .te()
                        .get(existing_trait_decl.implementing_for.type_id),
                    TypeInfo::UnknownGeneric { .. }
                )
            {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Visit)]
pub struct TyFunctionParameter {
    #[visit(skip)]
    pub name: Ident,
    #[visit(skip)]
    pub is_reference: bool,
    #[visit(skip)]
    pub is_mutable: bool,
    #[visit(skip)]
    pub mutability_span: Span,
    pub type_argument: GenericTypeArgument,
}

impl EqWithEngines for TyFunctionParameter {}
impl PartialEqWithEngines for TyFunctionParameter {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.type_argument.eq(&other.type_argument, ctx)
            && self.is_reference == other.is_reference
            && self.is_mutable == other.is_mutable
    }
}

impl HashWithEngines for TyFunctionParameter {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyFunctionParameter {
            name,
            is_reference,
            is_mutable,
            type_argument,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            mutability_span: _,
        } = self;
        name.hash(state);
        type_argument.hash(state, engines);
        is_reference.hash(state);
        is_mutable.hash(state);
    }
}

impl SubstTypes for TyFunctionParameter {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        self.type_argument.type_id.subst(ctx)
    }
}

impl TyFunctionParameter {
    pub fn is_self(&self) -> bool {
        self.name.as_str() == "self"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TyFunctionSigTypeParameter {
    Type(TypeId),
    Const(ConstGenericExpr),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TyFunctionSig {
    pub return_type: TypeId,
    pub parameters: Vec<TypeId>,
    pub type_parameters: Vec<TyFunctionSigTypeParameter>,
}

impl DisplayWithEngines for TyFunctionSig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{:?}", engines.help_out(self))
    }
}

impl DebugWithEngines for TyFunctionSig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let tp_str = if self.type_parameters.is_empty() {
            "".to_string()
        } else {
            format!(
                "<{}>",
                self.type_parameters
                    .iter()
                    .map(|p| match p {
                        TyFunctionSigTypeParameter::Type(t) => format!("{:?}", engines.help_out(t)),
                        TyFunctionSigTypeParameter::Const(expr) =>
                            format!("{:?}", engines.help_out(expr)),
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        };
        write!(
            f,
            "fn{}({}) -> {}",
            tp_str,
            self.parameters
                .iter()
                .map(|p| format!("{}", engines.help_out(p)))
                .collect::<Vec<_>>()
                .join(", "),
            engines.help_out(self.return_type),
        )
    }
}

impl TyFunctionSig {
    pub fn from_fn_decl(fn_decl: &TyFunctionDecl) -> Self {
        Self {
            return_type: fn_decl.return_type.type_id,
            parameters: fn_decl
                .parameters
                .iter()
                .map(|p| p.type_argument.type_id)
                .collect::<Vec<_>>(),
            type_parameters: fn_decl
                .type_parameters
                .iter()
                .map(|x| match x {
                    TypeParameter::Type(p) => TyFunctionSigTypeParameter::Type(p.type_id),
                    TypeParameter::Const(p) => {
                        let expr = ConstGenericExpr::AmbiguousVariableExpression {
                            ident: p.name.clone(),
                            decl: None,
                        };
                        TyFunctionSigTypeParameter::Const(p.expr.clone().unwrap_or(expr))
                    }
                })
                .collect(),
        }
    }

    pub fn is_concrete(&self, engines: &Engines) -> bool {
        self.return_type
            .is_concrete(engines, TreatNumericAs::Concrete)
            && self
                .parameters
                .iter()
                .all(|p| p.is_concrete(engines, TreatNumericAs::Concrete))
            && self.type_parameters.iter().all(|x| match x {
                TyFunctionSigTypeParameter::Type(type_id) => {
                    type_id.is_concrete(engines, TreatNumericAs::Concrete)
                }
                TyFunctionSigTypeParameter::Const(expr) => match expr {
                    ConstGenericExpr::Literal { .. } => true,
                    ConstGenericExpr::AmbiguousVariableExpression { .. } => false,
                },
            })
    }

    /// Returns a [String] representing the function.
    /// When the function is monomorphized the returned string is unique.
    /// Two monomorphized functions that generate the same string can be assumed to be the same.
    pub fn get_type_str(&self, engines: &Engines) -> String {
        let tp_str = if self.type_parameters.is_empty() {
            "".to_string()
        } else {
            format!(
                "<{}>",
                self.type_parameters
                    .iter()
                    .map(|x| match x {
                        TyFunctionSigTypeParameter::Type(type_id) => type_id.get_type_str(engines),
                        TyFunctionSigTypeParameter::Const(p) => {
                            match p {
                                ConstGenericExpr::Literal { val, .. } => val.to_string(),
                                ConstGenericExpr::AmbiguousVariableExpression { ident, .. } => {
                                    ident.as_str().to_string()
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        };
        format!(
            "fn{}({}) -> {}",
            tp_str,
            self.parameters
                .iter()
                .map(|p| p.get_type_str(engines))
                .collect::<Vec<_>>()
                .join(", "),
            self.return_type.get_type_str(engines),
        )
    }
}

// TODO: Investigate and fix the following invalid display:
//       - `<(Struct) as AbiDecode>::abi_decode(ref mut buffer: BufferReader)`
//         Note that sometimes it is properly displayed as
//         `<Struct as AbiDecode>::abi_decode(ref mut buffer: BufferReader)`

// TODO: Investigate why traits are sometimes not displayed with full path, e.g.:
//          `<path::Struct as Trait>::trait_method`
//       instead of
//          `<path::Struct as path::Trait>::trait_method`
//        Examples can be found in test:
//          should_fail/associated_type_multiple_traits_same_name/test.toml

// TODO: Investigate how to better display `Self` type in some edge cases,
//       like, e.g., in test:
//          should_fail/method_missing_constraint
//       It can be that this is not an issue of the type displaying but rather
//       the formed error itself.

/// Provides a configurable way to display a [TyFunctionDecl].
///
/// E.g., for a module function `some_function`:
/// - `some_function`
/// - `some_function(u64, T)`
/// - `some_function(u64, T) -> T`
/// - `some_pkg::some_module::some_function<T>(arg1: u64, arg2: T) -> T`
///
/// E.g., for a trait method `some_trait_method`:
/// - `some_lib::traits::MyTrait::some_trait_method(self: Self) -> u64`
/// - `<some_pkg::some_module::MyStruct<u64, bool> as some_lib::traits::MyTrait>::some_trait_method`
#[derive(Debug, Clone, Copy)]
pub struct TyFunctionDisplay {
    /// E.g., when true:
    /// - `SelfType::some_function`, if the function is declared in a trait or self impl.
    /// - `Trait::some_function`, or `Abi::some_function`, if it is a provided function.
    ///
    /// E.g., when false:
    /// - `some_function`.
    display_self_type: bool,
    /// E.g, when true: `<SelfType as Trait>::some_function`.
    /// E.g, when false: `SelfType::some_function`.
    display_trait: bool,
    /// E.g, when true: `some_pkg::some_module::some_module_function`.
    /// E.g, when false: `some_module_function`.
    display_module_fn_call_path: bool,
    /// E.g, when true: `some_function<A, B>`.
    /// E.g, when false: `some_function`.
    display_fn_type_params: bool,
    /// Display the type of the `self` parameter. E.g., `self: MyStruct<u64, bool>`.
    /// If false, it will just display `self`, if `display_param_names` is true.
    /// If `display_param_names` is false, it will still display the type name,
    /// if `display_param_types` is true.
    display_self_param_type: bool,
    /// E.g, when true: `some_function(ref mut a: u8)`, `some_function(ref mut a)`, or `some_function(ref mut u8)`.
    /// E.g, when false: `some_function(a: u8)`, `some_function(a)`, or `some_function(u8)`.
    display_ref_mut: bool,
    /// E.g, when true: `some_function(a: u8, b: u256)`.
    /// E.g, when false: `some_function(u8, u256)`.
    display_param_names: bool,
    /// E.g, when true: `some_function(a: u8, b: u256)`.
    /// E.g, when false: `some_function(a, b)`.
    display_param_types: bool,
    /// E.g, when true: `some_function -> ReturnType`.
    /// E.g, when false: `some_function`.
    display_return_type: bool,
    /// Defines how to display all of the types:
    /// - trait and self type,
    /// - type parameters,
    /// - self parameter type,
    /// - parameter types,
    /// - return type.
    types_display: TypeInfoDisplay,
}

impl TyFunctionDisplay {
    pub const fn only_name() -> Self {
        Self {
            display_trait: false,
            display_self_type: false,
            display_module_fn_call_path: false,
            display_fn_type_params: false,
            display_self_param_type: false,
            display_ref_mut: false,
            display_param_names: false,
            display_param_types: false,
            display_return_type: false,
            types_display: TypeInfoDisplay::only_name(),
        }
    }

    pub const fn full() -> Self {
        Self {
            display_trait: true,
            display_self_type: true,
            display_module_fn_call_path: true,
            display_fn_type_params: true,
            display_self_param_type: true,
            display_ref_mut: true,
            display_param_names: true,
            display_param_types: true,
            display_return_type: true,
            types_display: TypeInfoDisplay::full(),
        }
    }

    pub const fn with_trait(self) -> Self {
        Self {
            display_trait: true,
            ..self
        }
    }

    pub const fn without_trait(self) -> Self {
        Self {
            display_trait: false,
            ..self
        }
    }

    pub const fn with_self_type(self) -> Self {
        Self {
            display_self_type: true,
            ..self
        }
    }

    pub const fn without_self_type(self) -> Self {
        Self {
            display_self_type: false,
            ..self
        }
    }

    pub const fn with_module_fn_call_path(self) -> Self {
        Self {
            display_module_fn_call_path: true,
            ..self
        }
    }

    pub const fn without_module_fn_call_path(self) -> Self {
        Self {
            display_module_fn_call_path: false,
            ..self
        }
    }

    pub const fn with_fn_type_params(self) -> Self {
        Self {
            display_fn_type_params: true,
            ..self
        }
    }

    pub const fn without_fn_type_params(self) -> Self {
        Self {
            display_fn_type_params: false,
            ..self
        }
    }

    pub const fn with_self_param_type(self) -> Self {
        Self {
            display_self_param_type: true,
            ..self
        }
    }

    pub const fn without_self_param_type(self) -> Self {
        Self {
            display_self_param_type: false,
            ..self
        }
    }

    pub const fn with_ref_mut(self) -> Self {
        Self {
            display_ref_mut: true,
            ..self
        }
    }

    pub const fn without_ref_mut(self) -> Self {
        Self {
            display_ref_mut: false,
            ..self
        }
    }

    pub const fn with_param_names(self) -> Self {
        Self {
            display_param_names: true,
            ..self
        }
    }

    pub const fn without_param_names(self) -> Self {
        Self {
            display_param_names: false,
            ..self
        }
    }

    pub const fn with_param_types(self) -> Self {
        Self {
            display_param_types: true,
            ..self
        }
    }

    pub const fn without_param_types(self) -> Self {
        Self {
            display_param_types: false,
            ..self
        }
    }

    pub const fn with_return_type(self) -> Self {
        Self {
            display_return_type: true,
            ..self
        }
    }

    pub const fn without_return_type(self) -> Self {
        Self {
            display_return_type: false,
            ..self
        }
    }

    pub const fn with_types_display(self, types_display: TypeInfoDisplay) -> Self {
        Self {
            types_display,
            ..self
        }
    }

    pub const fn with_signature(self) -> Self {
        Self {
            display_param_names: true,
            display_param_types: true,
            display_return_type: true,
            ..self
        }
    }

    pub const fn without_signature(self) -> Self {
        Self {
            display_param_names: false,
            display_param_types: false,
            display_return_type: false,
            ..self
        }
    }

    pub const fn with_parameters(self) -> Self {
        Self {
            display_param_names: true,
            display_param_types: true,
            ..self
        }
    }

    pub const fn without_parameters(self) -> Self {
        Self {
            display_param_names: false,
            display_param_types: false,
            ..self
        }
    }

    fn should_display_parameters(&self) -> bool {
        self.display_param_names || self.display_param_types
    }

    fn should_display_param_type(&self, param: &TyFunctionParameter) -> bool {
        self.display_param_types
            && (param.is_self() && self.display_self_param_type || !param.is_self())
    }

    fn is_module_function(&self, fn_decl: &TyFunctionDecl) -> bool {
        fn_decl.implementing_type.is_none() && fn_decl.implementing_for.is_none()
    }

    /// Quick heuristic to calculate the initial capacity of the [String]
    /// used to store the display of the function represented by `fn_decl`.
    fn calculate_initial_string_capacity(&self, fn_decl: &TyFunctionDecl) -> usize {
        const DEFAULT_TYPE_NAME_LENGTH: usize = 10;
        const DEFAULT_CONST_GENERIC_TYPE_PARAM_LENGTH: usize = 2; // E.g., `T`, or `42`.
        const DOUBLE_COLON_LENGTH: usize = 2;

        let mut capacity = 0;

        if (self.display_trait || self.display_self_type) && fn_decl.implementing_type.is_some() {
            capacity += DEFAULT_TYPE_NAME_LENGTH + DOUBLE_COLON_LENGTH;
        }

        capacity += fn_decl.name.as_str().len();
        // If it's a module function and we need to display the call path.
        if self.display_module_fn_call_path && self.is_module_function(fn_decl) {
            capacity += fn_decl.call_path.prefixes.iter().fold(0, |acc, prefix| {
                acc + prefix.as_str().len() + DOUBLE_COLON_LENGTH
            });
        }

        if self.display_fn_type_params && !fn_decl.type_parameters.is_empty() {
            capacity += 2; // For angle brackets.
            capacity += fn_decl.type_parameters.iter().fold(0, |acc, tp| {
                acc + match tp {
                    TypeParameter::Type(_) => DEFAULT_TYPE_NAME_LENGTH,
                    TypeParameter::Const(_) => DEFAULT_CONST_GENERIC_TYPE_PARAM_LENGTH,
                } + 2 // For the type parameter name and the comma.
            });
        }

        if self.should_display_parameters() {
            capacity += 2; // For parentheses.

            fn_decl.parameters.iter().for_each(|param| {
                if self.display_param_names {
                    capacity += param.name.as_str().len();
                    if self.should_display_param_type(param) {
                        capacity += 2; // For the colon and space `: `.
                    }
                }
                if self.should_display_param_type(param) {
                    capacity += DEFAULT_TYPE_NAME_LENGTH;
                }

                capacity += 2; // For the comma and space `, `.
            });

            if !fn_decl.parameters.is_empty() {
                capacity -= 2; // Remove the last comma and space `, `.
            }
        }

        if self.display_return_type {
            capacity += 4; // For the ` -> `.
            capacity += DEFAULT_TYPE_NAME_LENGTH;
        }

        capacity
    }

    pub fn display(&self, fn_decl: &TyFunctionDecl, engines: &Engines) -> String {
        let mut result = String::with_capacity(self.calculate_initial_string_capacity(fn_decl));

        // Append call path to module function, or self type and trait type to members,
        // if configured so.
        if self.display_module_fn_call_path && self.is_module_function(fn_decl) {
            // TODO: Remove this workaround once https://github.com/FuelLabs/sway/issues/7304 is fixed
            //       and uncomment the original code below.
            if let Some((first_prefix, rest_prefixes)) = fn_decl.call_path.prefixes.split_first() {
                let first_prefix = if !first_prefix.as_str().contains('-') {
                    first_prefix.as_str()
                } else {
                    &first_prefix.as_str().replace('-', "_")
                };
                result.push_str(first_prefix);
                result.push_str("::");
                for prefix in rest_prefixes {
                    result.push_str(prefix.as_str());
                    result.push_str("::");
                }
            }

            // fn_decl.call_path.prefixes.iter().for_each(|prefix| {
            //     result.push_str(prefix.as_str());
            //     result.push_str("::");
            // });
        } else if self.display_self_type || self.display_trait {
            match fn_decl.implementing_type.as_ref() {
                Some(TyDecl::TraitDecl(trait_decl)) if self.display_self_type => {
                    // The function is a provided trait function, so in the context of displaying,
                    // we treat the trait as the self type.
                    let trait_decl = engines.de().get_trait(&trait_decl.decl_id);
                    self.display_udt_decl_into(
                        &trait_decl.call_path,
                        Either::Left(&trait_decl.type_parameters),
                        engines,
                        &mut result,
                    );
                    result.push_str("::");
                }
                Some(TyDecl::AbiDecl(abi_decl)) if self.display_self_type => {
                    // The function is a provided ABI function, so in the context of displaying,
                    // we treat the ABI as the self type.
                    let abi_decl = engines.de().get_abi(&abi_decl.decl_id);
                    // TODO: Add call path support for `TyAbiDecl`. Currently, it contains only the name.
                    //       When done, call `self.display_udt_decl_into` here, with empty type parameters.
                    result.push_str(abi_decl.name.as_str());
                    result.push_str("::");
                }
                Some(TyDecl::ImplSelfOrTrait(impl_self_or_trait_decl)) => {
                    let impl_self_or_trait_decl = engines
                        .de()
                        .get_impl_self_or_trait(&impl_self_or_trait_decl.decl_id);
                    let self_type = if self.display_self_type {
                        let implementing_for = match fn_decl.implementing_for {
                            Some(implementing_for) => engines.te().get(implementing_for),
                            None => {
                                // No implementing for provided, as a fallback we use the one
                                // from the `impl_self_or_trait_decl`.
                                engines
                                    .te()
                                    .get(impl_self_or_trait_decl.implementing_for.type_id)
                            }
                        };
                        Some(
                            self.types_display
                                .display(&implementing_for, engines)
                                .to_string(),
                        )
                    } else {
                        None
                    };
                    let trait_type = if self.display_trait {
                        impl_self_or_trait_decl
                            .as_ref()
                            .trait_decl_ref
                            .as_ref()
                            .map(|trait_or_abi_decl| {
                                match trait_or_abi_decl.id() {
                                    InterfaceDeclId::Abi(decl_id) => {
                                        let abi_decl = engines.de().get_abi(decl_id);
                                        abi_decl.name.to_string()
                                    }
                                    InterfaceDeclId::Trait(decl_id) => {
                                        let trait_decl = engines.de().get_trait(decl_id);
                                        // Take the trait call path from the declaration,
                                        // and the actual parameters from the impl.
                                        self.display_udt_decl(
                                            &trait_decl.call_path,
                                            Either::Right(
                                                &impl_self_or_trait_decl.trait_type_arguments,
                                            ),
                                            engines,
                                        )
                                    }
                                }
                            })
                    } else {
                        None
                    };

                    match (self_type, trait_type) {
                        (None, None) => {}
                        (None, Some(type_name)) | (Some(type_name), None) => {
                            result.push_str(&type_name);
                            result.push_str("::");
                        }
                        (Some(self_type), Some(trait_type)) => {
                            result.push('<');
                            result.push_str(&self_type);
                            result.push_str(" as ");
                            result.push_str(&trait_type);
                            result.push('>');
                            result.push_str("::");
                        }
                    }
                }
                _ => {
                    if let Some(implementing_for) = fn_decl.implementing_for {
                        let implementing_for = engines.te().get(implementing_for);
                        result.push_str(&self.types_display.display(&implementing_for, engines));
                        result.push_str("::");
                    }
                }
            }
        }

        // Always append function name.
        result.push_str(fn_decl.name.as_str());

        // Append function parameters, if configured so.
        if self.should_display_parameters() {
            result.push('(');

            fn_decl.parameters.iter().for_each(|param| {
                if self.display_ref_mut && param.is_mutable && param.is_reference {
                    result.push_str("ref mut ");
                }
                if self.display_param_names {
                    result.push_str(param.name.as_str());
                    if self.should_display_param_type(param) {
                        result.push_str(": ");
                    }
                }
                if self.should_display_param_type(param) {
                    let param_type = engines.te().get(param.type_argument.type_id);
                    result.push_str(&self.types_display.display(&param_type, engines));
                }

                result.push_str(", ");
            });

            // Remove trailing comma and space if present.
            result.truncate(result.rfind(',').unwrap_or(result.len()));

            result.push(')');
        }

        // Append return type, if configured so.
        if self.display_return_type {
            result.push_str(" -> ");
            let return_type = engines.te().get(fn_decl.return_type.type_id);
            result.push_str(&self.types_display.display(&return_type, engines));
        }

        result
    }

    fn display_udt_decl(
        &self,
        udt_name: &CallPath,
        type_params: Either<&[TypeParameter], &[GenericArgument]>,
        engines: &Engines,
    ) -> String {
        let capacity = udt_name.suffix.as_str().len()
            + if self.types_display.display_call_paths {
                udt_name
                    .prefixes
                    .iter()
                    .map(|p| p.as_str().len())
                    .sum::<usize>()
            } else {
                0
            };

        let mut dest = String::with_capacity(capacity);
        self.display_udt_decl_into(udt_name, type_params, engines, &mut dest);
        dest
    }

    /// Displays a user-defined type (UDT) declaration into the `dest`.
    /// UDTs are: traits, ABIs, structs, enums, and type aliases.
    fn display_udt_decl_into(
        &self,
        udt_name: &CallPath,
        type_params: Either<&[TypeParameter], &[GenericArgument]>,
        engines: &Engines,
        dest: &mut String,
    ) {
        if self.types_display.display_call_paths {
            dest.push_str(&udt_name.to_string());
        } else {
            dest.push_str(udt_name.suffix.as_str());
        }

        match type_params {
            Either::Left(type_params) => {
                if !type_params.is_empty() {
                    dest.push_str(
                        &self
                            .types_display
                            .display_non_empty_type_params(type_params, engines),
                    );
                }
            }
            Either::Right(generic_args) => {
                if !generic_args.is_empty() {
                    dest.push_str(
                        &self
                            .types_display
                            .display_non_empty_generic_args(generic_args, engines),
                    );
                }
            }
        }
    }
}
