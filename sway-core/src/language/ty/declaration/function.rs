use std::hash::{Hash, Hasher};

use sha2::{Digest, Sha256};

use crate::{
    decl_engine::*,
    engine_threading::*,
    error::*,
    language::{parsed, ty::*, Inline, Purity, Visibility},
    transform,
    type_system::*,
};

use sway_types::{
    constants::{INLINE_ALWAYS_NAME, INLINE_NEVER_NAME},
    Ident, Named, Span, Spanned,
};

#[derive(Clone, Debug)]
pub struct TyFunctionDeclaration {
    pub name: Ident,
    pub body: TyCodeBlock,
    pub parameters: Vec<TyFunctionParameter>,
    pub implementing_type: Option<TyDeclaration>,
    pub span: Span,
    pub attributes: transform::AttributesMap,
    pub type_parameters: Vec<TypeParameter>,
    pub return_type: TypeArgument,
    pub visibility: Visibility,
    /// whether this function exists in another contract and requires a call to it or not
    pub is_contract_call: bool,
    pub purity: Purity,
    pub where_clause: Vec<(Ident, Vec<TraitConstraint>)>,
}

impl Named for TyFunctionDeclaration {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl EqWithEngines for TyFunctionDeclaration {}
impl PartialEqWithEngines for TyFunctionDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
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

impl HashWithEngines for TyFunctionDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyFunctionDeclaration {
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
            span: _,
            attributes: _,
            implementing_type: _,
            where_clause: _,
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

impl SubstTypes for TyFunctionDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
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

impl ReplaceSelfType for TyFunctionDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.return_type.replace_self_type(engines, self_type);
        self.body.replace_self_type(engines, self_type);
    }
}

impl ReplaceDecls for TyFunctionDeclaration {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        self.body.replace_decls(decl_mapping, engines);
    }
}

impl Spanned for TyFunctionDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyFunctionDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }
}

impl UnconstrainedTypeParameters for TyFunctionDeclaration {
    fn type_parameter_is_unconstrained(
        &self,
        engines: Engines<'_>,
        type_parameter: &TypeParameter,
    ) -> bool {
        let type_engine = engines.te();
        let type_parameter_info = type_engine.get(type_parameter.type_id);
        if self
            .type_parameters
            .iter()
            .map(|type_param| type_engine.get(type_param.type_id))
            .any(|x| x.eq(&type_parameter_info, engines))
        {
            return false;
        }
        if self
            .parameters
            .iter()
            .map(|param| type_engine.get(param.type_argument.type_id))
            .any(|x| x.eq(&type_parameter_info, engines))
        {
            return true;
        }
        if type_engine
            .get(self.return_type.type_id)
            .eq(&type_parameter_info, engines)
        {
            return true;
        }

        false
    }
}

impl CollectTypesMetadata for TyFunctionDeclaration {
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut body = vec![];
        for content in self.body.contents.iter() {
            body.append(&mut check!(
                content.collect_types_metadata(ctx),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }
        body.append(&mut check!(
            self.return_type.type_id.collect_types_metadata(ctx),
            return err(warnings, errors),
            warnings,
            errors
        ));
        for type_param in self.type_parameters.iter() {
            body.append(&mut check!(
                type_param.type_id.collect_types_metadata(ctx),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }
        for param in self.parameters.iter() {
            body.append(&mut check!(
                param.type_argument.type_id.collect_types_metadata(ctx),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }
        ok(body, warnings, errors)
    }
}

impl TyFunctionDeclaration {
    pub(crate) fn set_implementing_type(&mut self, decl: TyDeclaration) {
        self.implementing_type = Some(decl);
    }

    /// Used to create a stubbed out function when the function fails to
    /// compile, preventing cascading namespace errors.
    pub(crate) fn error(decl: parsed::FunctionDeclaration) -> TyFunctionDeclaration {
        let parsed::FunctionDeclaration {
            name,
            return_type,
            span,
            visibility,
            purity,
            where_clause,
            ..
        } = decl;
        TyFunctionDeclaration {
            purity,
            name,
            body: TyCodeBlock {
                contents: Default::default(),
            },
            implementing_type: None,
            span,
            attributes: Default::default(),
            is_contract_call: false,
            parameters: Default::default(),
            visibility,
            return_type,
            type_parameters: Default::default(),
            where_clause,
        }
    }

    /// If there are parameters, join their spans. Otherwise, use the fn name span.
    pub(crate) fn parameters_span(&self) -> Span {
        if !self.parameters.is_empty() {
            self.parameters.iter().fold(
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
        type_engine: &TypeEngine,
    ) -> CompileResult<Vec<u8>> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut hasher = Sha256::new();
        let data = check!(
            self.to_selector_name(type_engine),
            return err(warnings, errors),
            warnings,
            errors
        );
        hasher.update(data);
        let hash = hasher.finalize();
        ok(hash.to_vec(), warnings, errors)
    }

    /// Converts a [TyFunctionDeclaration] into a value that is to be used in contract function
    /// selectors.
    /// Hashes the name and parameters using SHA256, and then truncates to four bytes.
    pub fn to_fn_selector_value(&self, type_engine: &TypeEngine) -> CompileResult<[u8; 4]> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let hash = check!(
            self.to_fn_selector_value_untruncated(type_engine),
            return err(warnings, errors),
            warnings,
            errors
        );
        // 4 bytes truncation via copying into a 4 byte buffer
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&hash[..4]);
        ok(buf, warnings, errors)
    }

    pub fn to_selector_name(&self, type_engine: &TypeEngine) -> CompileResult<String> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let named_params = self
            .parameters
            .iter()
            .map(|TyFunctionParameter { type_argument, .. }| {
                type_engine
                    .to_typeinfo(type_argument.type_id, &type_argument.span)
                    .expect("unreachable I think?")
                    .to_selector_name(type_engine, &type_argument.span)
            })
            .filter_map(|name| name.ok(&mut warnings, &mut errors))
            .collect::<Vec<String>>();

        ok(
            format!("{}({})", self.name.as_str(), named_params.join(","),),
            warnings,
            errors,
        )
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
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.type_argument.eq(&other.type_argument, engines)
            && self.is_reference == other.is_reference
            && self.is_mutable == other.is_mutable
    }
}

impl HashWithEngines for TyFunctionParameter {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
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
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_argument.type_id.subst(type_mapping, engines);
    }
}

impl ReplaceSelfType for TyFunctionParameter {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_argument
            .type_id
            .replace_self_type(engines, self_type);
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
    pub fn from_fn_decl(fn_decl: &TyFunctionDeclaration) -> Self {
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
