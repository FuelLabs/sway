use std::{
    collections::HashSet,
    fmt,
    hash::{Hash, Hasher},
};

use sha2::{Digest, Sha256};
use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::CallPath, semantic_analysis::type_check_context::MonomorphizeHelper};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::{parsed, ty::*, Inline, Purity, Visibility},
    semantic_analysis::TypeCheckContext,
    transform,
    type_system::*,
    types::*,
};

use sway_types::{
    constants::{INLINE_ALWAYS_NAME, INLINE_NEVER_NAME},
    Ident, Named, Span, Spanned,
};

#[derive(Clone, Debug)]
pub struct TyFunctionDecl {
    pub name: Ident,
    pub body: TyCodeBlock,
    pub parameters: Vec<TyFunctionParameter>,
    pub implementing_type: Option<TyDecl>,
    pub span: Span,
    pub call_path: CallPath,
    pub attributes: transform::AttributesMap,
    pub type_parameters: Vec<TypeParameter>,
    pub return_type: TypeArgument,
    pub visibility: Visibility,
    /// whether this function exists in another contract and requires a call to it or not
    pub is_contract_call: bool,
    pub purity: Purity,
    pub where_clause: Vec<(Ident, Vec<TraitConstraint>)>,
    pub is_trait_method_dummy: bool,
}

impl DebugWithEngines for TyFunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "{}{:?}{}({}):{}",
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
                        .map(|p| format!("{:?}", engines.help_out(p.initial_type_id)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                "".to_string()
            },
            self.parameters
                .iter()
                .map(|p| format!(
                    "{}:{}",
                    p.name.as_str(),
                    engines.help_out(p.type_argument.initial_type_id)
                ))
                .collect::<Vec<_>>()
                .join(", "),
            engines.help_out(self.return_type.initial_type_id),
        )
    }
}

impl Named for TyFunctionDecl {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl declaration::FunctionSignature for TyFunctionDecl {
    fn parameters(&self) -> &Vec<TyFunctionParameter> {
        &self.parameters
    }

    fn return_type(&self) -> &TypeArgument {
        &self.return_type
    }
}

impl EqWithEngines for TyFunctionDecl {}
impl PartialEqWithEngines for TyFunctionDecl {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.name == other.name
            && self.body.eq(&other.body, engines)
            && self.parameters.eq(&other.parameters, engines)
            && self.return_type.eq(&other.return_type, engines)
            && self.type_parameters.eq(&other.type_parameters, engines)
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
            where_clause: _,
            is_trait_method_dummy: _,
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
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.return_type.subst(type_mapping, engines);
        self.body.subst(type_mapping, engines);
    }
}

impl ReplaceDecls for TyFunctionDecl {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        self.body.replace_decls(decl_mapping, handler, ctx)
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

impl UnconstrainedTypeParameters for TyFunctionDecl {
    fn type_parameter_is_unconstrained(
        &self,
        engines: &Engines,
        type_parameter: &TypeParameter,
    ) -> bool {
        let type_engine = engines.te();
        let mut all_types: HashSet<TypeId> = self
            .type_parameters
            .iter()
            .map(|type_param| type_param.type_id)
            .collect();
        all_types.extend(self.parameters.iter().flat_map(|param| {
            let mut inner = param.type_argument.type_id.extract_inner_types(engines);
            inner.insert(param.type_argument.type_id);
            inner
        }));
        all_types.extend(self.return_type.type_id.extract_inner_types(engines));
        all_types.insert(self.return_type.type_id);
        let type_parameter_info = type_engine.get(type_parameter.type_id);
        all_types
            .iter()
            .any(|type_id| type_engine.get(*type_id).eq(&type_parameter_info, engines))
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
        for type_param in self.type_parameters.iter() {
            body.append(&mut type_param.type_id.collect_types_metadata(handler, ctx)?);
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
    pub(crate) fn error(decl: parsed::FunctionDeclaration) -> TyFunctionDecl {
        let parsed::FunctionDeclaration {
            name,
            return_type,
            span,
            visibility,
            purity,
            where_clause,
            ..
        } = decl;
        TyFunctionDecl {
            purity,
            name,
            body: TyCodeBlock::default(),
            implementing_type: None,
            span,
            call_path: CallPath::from(Ident::dummy()),
            attributes: Default::default(),
            is_contract_call: false,
            parameters: Default::default(),
            visibility,
            return_type,
            type_parameters: Default::default(),
            where_clause,
            is_trait_method_dummy: false,
        }
    }

    /// If there are parameters, join their spans. Otherwise, use the fn name span.
    pub(crate) fn parameters_span(&self) -> Span {
        if !self.parameters.is_empty() {
            self.parameters.iter().fold(
                // TODO: Use Span::join_all().
                self.parameters[0].name.span(),
                |acc, TyFunctionParameter { type_argument, .. }| {
                    Span::join(acc, type_argument.span.clone())
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

    /// Whether or not this function is the default entry point.
    pub fn is_main_entry(&self) -> bool {
        // NOTE: We may want to make this check more sophisticated or customisable in the future,
        // but for now this assumption is baked in throughout the compiler.
        self.name.as_str() == sway_types::constants::DEFAULT_ENTRY_POINT_FN_NAME
    }

    /// Whether or not this function is a unit test, i.e. decorated with `#[test]`.
    pub fn is_test(&self) -> bool {
        self.attributes
            .contains_key(&transform::AttributeKind::Test)
    }

    pub fn inline(&self) -> Option<Inline> {
        match self
            .attributes
            .get(&transform::AttributeKind::Inline)?
            .last()?
            .args
            .first()?
            .name
            .as_str()
        {
            INLINE_NEVER_NAME => Some(Inline::Never),
            INLINE_ALWAYS_NAME => Some(Inline::Always),
            _ => None,
        }
    }

    /// Whether or not this function describes a program entry point.
    pub fn is_entry(&self) -> bool {
        self.is_main_entry() || self.is_test()
    }
}

#[derive(Debug, Clone)]
pub struct TyFunctionParameter {
    pub name: Ident,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub mutability_span: Span,
    pub type_argument: TypeArgument,
}

impl EqWithEngines for TyFunctionParameter {}
impl PartialEqWithEngines for TyFunctionParameter {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.name == other.name
            && self.type_argument.eq(&other.type_argument, engines)
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
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        self.type_argument.type_id.subst(type_mapping, engines);
    }
}

impl TyFunctionParameter {
    pub fn is_self(&self) -> bool {
        self.name.as_str() == "self"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TyFunctionSig {
    pub return_type: TypeId,
    pub parameters: Vec<TypeId>,
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
        }
    }
}
