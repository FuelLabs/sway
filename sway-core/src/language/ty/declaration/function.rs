use crate::{
    decl_engine::*,
    engine_threading::*,
    has_changes,
    language::{
        parsed::{self, FunctionDeclaration, FunctionDeclarationKind},
        ty::*,
        CallPath, Inline, Purity, Visibility,
    },
    semantic_analysis::TypeCheckContext,
    transform::{self, AttributeKind},
    type_system::*,
    types::*,
};
use ast_elements::type_parameter::ConstGenericExpr;
use monomorphization::MonomorphizeHelper;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt,
    hash::{Hash, Hasher},
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TyFunctionDeclKind {
    Default,
    Entry,
    Main,
    Test,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyFunctionDecl {
    pub name: Ident,
    pub body: TyCodeBlock,
    pub parameters: Vec<TyFunctionParameter>,
    pub implementing_type: Option<TyDecl>,
    pub implementing_for_typeid: Option<TypeId>,
    pub span: Span,
    pub call_path: CallPath,
    pub attributes: transform::Attributes,
    pub type_parameters: Vec<TypeParameter>,
    pub return_type: GenericArgument,
    pub visibility: Visibility,
    /// whether this function exists in another contract and requires a call to it or not
    pub is_contract_call: bool,
    pub purity: Purity,
    pub where_clause: Vec<(Ident, Vec<TraitConstraint>)>,
    pub is_trait_method_dummy: bool,
    pub is_type_check_finalized: bool,
    pub kind: TyFunctionDeclKind,
}

impl TyDeclParsedType for TyFunctionDecl {
    type ParsedType = FunctionDeclaration;
}

impl DebugWithEngines for TyFunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "{}{:?}{}({}):{}->{}",
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
                        .map(|p| {
                            match p {
                                TypeParameter::Type(p) => {
                                    format!(
                                        "{:?} -> {:?}",
                                        engines.help_out(p.initial_type_id),
                                        engines.help_out(p.type_id)
                                    )
                                }
                                TypeParameter::Const(p) => {
                                    let decl = engines.de().get(p.tid.id());
                                    format!("{} -> {:?}", decl.name(), p.expr)
                                }
                            }
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
                    "{}:{} -> {}",
                    p.name.as_str(),
                    engines.help_out(p.type_argument.initial_type_id()),
                    engines.help_out(p.type_argument.type_id())
                ))
                .collect::<Vec<_>>()
                .join(", "),
            engines.help_out(self.return_type.initial_type_id()),
            engines.help_out(self.return_type.type_id()),
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
                    engines.help_out(p.type_argument.initial_type_id())
                ))
                .collect::<Vec<_>>()
                .join(", "),
            engines.help_out(self.return_type.initial_type_id()),
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
                TypeParameter::Const(p) => {
                    assert!(p.expr.is_none());

                    let decl = engines.de().get(p.tid.id());
                    if decl.name().as_str() == name {
                        p.expr = Some(ConstGenericExpr::from_ty_expression(handler, value)?);
                    }
                }
                _ => {}
            }
        }

        for param in self.parameters.iter_mut() {
            param
                .type_argument
                .type_id_mut()
                .materialize_const_generics(engines, handler, name, value)?;
        }
        self.return_type
            .type_id_mut()
            .materialize_const_generics(engines, handler, name, value)?;
        self.body
            .materialize_const_generics(engines, handler, name, value)
    }
}

impl DeclRefFunction {
    /// Makes method with a copy of type_id.
    /// This avoids altering the type_id already in the type map.
    /// Without this it is possible to retrieve a method from the type map unify its types and
    /// the second time it won't be possible to retrieve the same method.
    pub fn get_method_safe_to_unify(&self, engines: &Engines, type_id: TypeId) -> Self {
        let decl_engine = engines.de();

        let original = &*decl_engine.get_function(self);
        let mut method = original.clone();

        if let Some(method_implementing_for_typeid) = method.implementing_for_typeid {
            let mut type_id_type_subst_map = TypeSubstMap::new();

            if let Some(TyDecl::ImplSelfOrTrait(t)) = &method.implementing_type {
                let impl_self_or_trait = &*engines.de().get(&t.decl_id);

                let mut type_id_type_parameters = vec![];
                let mut const_generic_parameters = BTreeMap::default();
                type_id.extract_type_parameters(
                    engines,
                    0,
                    &mut type_id_type_parameters,
                    &mut const_generic_parameters,
                    impl_self_or_trait.implementing_for.type_id(),
                );

                eprintln!("get_method_safe_to_unify: {:?} -> {:?}",
                    engines.help_out(type_id),
                    engines.help_out(impl_self_or_trait.implementing_for.type_id())
                );
                for (k, v) in type_id_type_parameters.iter() {
                    eprintln!("    {:?} -> {:?};", engines.help_out(k), engines.help_out(v));
                }

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
                        .get(impl_self_or_trait.implementing_for.initial_type_id())
                        .eq(
                            &*engines.te().get(p.initial_type_id),
                            &PartialEqWithEnginesContext::new(engines),
                        )
                    {
                        type_id_type_subst_map.insert(p.type_id, type_id);
                    }
                }
            }

            let mut method_type_subst_map = TypeSubstMap::new();

            // Duplicate arguments to avoid changing TypeId inside TraitMap
            for parameter in method.parameters.iter_mut() {
                let old_id = parameter.type_argument.type_id();
                let new_id = engines
                    .te()
                    .duplicate(engines, old_id);
                method_type_subst_map.insert(old_id, new_id);
            }

            method_type_subst_map.insert(method.return_type.type_id(), engines
                .te()
                .duplicate(engines, method.return_type.type_id())
            );

            for p in method.type_parameters.iter() {
                match p {
                    TypeParameter::Type(_) => {},
                    TypeParameter::Const(p) => {
                        let old_id = p.tid.id();
                        let new_id = engines.de().duplicate(old_id);
                        method_type_subst_map.insert_const_generics(*old_id, *new_id.id());
                    },
                }
            }

            method_type_subst_map.extend(&type_id_type_subst_map);
            method_type_subst_map.insert(method_implementing_for_typeid, type_id);

            method.subst(&SubstTypesContext::new(
                engines,
                &method_type_subst_map,
                true,
            ));

            return engines
                .de()
                .insert(
                    method.clone(),
                    engines.de().get_parsed_decl_id(self.id()).as_ref(),
                )
                .with_parent(decl_engine, self.id().into());
        }

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
                .type_id()
                .is_concrete(engines, TreatNumericAs::Concrete)
            && self.parameters().iter().all(|t| {
                t.type_argument
                    .type_id()
                    .is_concrete(engines, TreatNumericAs::Concrete)
            })
    }
}
impl declaration::FunctionSignature for TyFunctionDecl {
    fn parameters(&self) -> &Vec<TyFunctionParameter> {
        &self.parameters
    }

    fn return_type(&self) -> &GenericArgument {
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
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            call_path: _,
            span: _,
            attributes: _,
            implementing_type: _,
            implementing_for_typeid: _,
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
    }
}

impl SubstTypes for TyFunctionDecl {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        if self.name.as_str().contains("abi_decode") {
            eprintln!("subst_inner: {:?}", ctx.engines.help_out(ctx.type_subst_map));
        }

        let changes = if ctx.subst_function_body {
            has_changes! {
                self.type_parameters.subst(ctx);
                self.parameters.subst(ctx);
                self.return_type.subst(ctx);
                self.body.subst(ctx);
                self.implementing_for_typeid.subst(ctx);
            }
        } else {
            has_changes! {
                self.type_parameters.subst(ctx);
                self.parameters.subst(ctx);
                self.return_type.subst(ctx);
                self.implementing_for_typeid.subst(ctx);
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
        let mut func_ctx = ctx.by_ref().with_self_type(self.implementing_for_typeid);
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
                .type_id()
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
                    .type_id()
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
            implementing_for_typeid: None,
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
                    Span::join(acc, &type_argument.span())
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
                    .to_typeinfo(type_argument.type_id(), &type_argument.span())
                    .expect("unreachable I think?")
                    .to_selector_name(handler, engines, &type_argument.span())
            })
            .filter_map(|name| name.ok())
            .collect::<Vec<String>>();

        Ok(format!(
            "{}({})",
            self.name.as_str(),
            named_params.join(","),
        ))
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
        //TODO match kind to Test
        self.attributes.has_any_of_kind(AttributeKind::Test)
    }

    pub fn inline(&self) -> Option<Inline> {
        self.attributes.inline()
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

                let implementing_for = engines.de().get(&t.decl_id).implementing_for.type_id();

                // TODO: Implement the check in detail for all possible cases (e.g. trait impls for generics etc.)
                //       and return just the definite `bool` and not `Option<bool>`.
                //       That would be too much effort at the moment for the immediate practical need of
                //       error reporting where we suggest obvious most common constructors
                //       that will be found using this simple check.
                if unify_check.check(type_id, implementing_for)
                    && unify_check.check(type_id, self.return_type.type_id())
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
        if let Some(TyDecl::ImplSelfOrTrait(existing_impl_trait)) = self.implementing_type.clone() {
            let existing_trait_decl = engines
                .de()
                .get_impl_self_or_trait(&existing_impl_trait.decl_id);
            if !existing_trait_decl.impl_type_parameters.is_empty()
                && matches!(
                    *engines
                        .te()
                        .get(existing_trait_decl.implementing_for.type_id()),
                    TypeInfo::UnknownGeneric { .. }
                )
            {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TyFunctionParameter {
    pub name: Ident,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub mutability_span: Span,
    pub type_argument: GenericArgument,
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
        self.type_argument.type_id_mut().subst(ctx)
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
            return_type: fn_decl.return_type.type_id(),
            parameters: fn_decl
                .parameters
                .iter()
                .map(|p| p.type_argument.type_id())
                .collect::<Vec<_>>(),
            type_parameters: fn_decl
                .type_parameters
                .iter()
                .map(|x| match x {
                    TypeParameter::Type(p) => TyFunctionSigTypeParameter::Type(p.type_id),
                    TypeParameter::Const(p) => {
                        let expr = ConstGenericExpr::AmbiguousVariableExpression {
                            ident: p.tid.name().clone(),
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
            && self
                .type_parameters
                .iter()
                .filter_map(|x| match x {
                    TyFunctionSigTypeParameter::Type(type_id) => Some(type_id),
                    TyFunctionSigTypeParameter::Const(_) => None,
                })
                .all(|type_id| type_id.is_concrete(engines, TreatNumericAs::Concrete))
    }

    /// Returns a String representing the function.
    /// When the function is monomorphized the returned String is unique.
    /// Two monomorphized functions that generate the same String can be assumed to be the same.
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
                                ConstGenericExpr::AmbiguousVariableExpression { ident } => {
                                    ident.as_str().to_string()
                                }
                                _ => todo!(),
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
