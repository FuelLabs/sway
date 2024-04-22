use std::{
    fmt,
    hash::{Hash, Hasher},
};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    engine_threading::*,
    has_changes,
    language::{ty::*, Purity},
    semantic_analysis::type_check_context::MonomorphizeHelper,
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyTraitFn {
    pub name: Ident,
    pub(crate) span: Span,
    pub(crate) purity: Purity,
    pub parameters: Vec<TyFunctionParameter>,
    pub return_type: TypeArgument,
    pub attributes: transform::AttributesMap,
}

impl DebugWithEngines for TyTraitFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "{:?}({}):{}",
            self.name,
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

impl Named for TyTraitFn {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Spanned for TyTraitFn {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl declaration::FunctionSignature for TyTraitFn {
    fn parameters(&self) -> &Vec<TyFunctionParameter> {
        &self.parameters
    }

    fn return_type(&self) -> &TypeArgument {
        &self.return_type
    }
}

impl EqWithEngines for TyTraitFn {}
impl PartialEqWithEngines for TyTraitFn {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        self.name == other.name
            && self.purity == other.purity
            && self.parameters.eq(&other.parameters, ctx)
            && type_engine
                .get(self.return_type.type_id)
                .eq(&type_engine.get(other.return_type.type_id), ctx)
            && self.attributes == other.attributes
    }
}

impl HashWithEngines for TyTraitFn {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyTraitFn {
            name,
            purity,
            parameters,
            return_type,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        let type_engine = engines.te();
        name.hash(state);
        parameters.hash(state, engines);
        type_engine.get(return_type.type_id).hash(state, engines);
        purity.hash(state);
    }
}

impl SubstTypes for TyTraitFn {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        has_changes! {
            self.parameters.subst(type_mapping, engines);
            self.return_type.subst(type_mapping, engines);
        }
    }
}

impl MonomorphizeHelper for TyTraitFn {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn type_parameters(&self) -> &[TypeParameter] {
        &[]
    }

    fn has_self_type_param(&self) -> bool {
        false
    }
}
