use crate::{
    engine_threading::*,
    language::{parsed::*, *},
    transform::{self, AttributeKind},
    type_system::*,
};
use sway_types::{ident::Ident, span::Span, Named, Spanned};

#[derive(Debug, Clone)]
pub enum FunctionDeclarationKind {
    Default,
    Entry,
    Main,
    Test,
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub purity: Purity,
    pub attributes: transform::AttributesMap,
    pub name: Ident,
    pub visibility: Visibility,
    pub body: CodeBlock,
    pub parameters: Vec<FunctionParameter>,
    pub span: Span,
    pub return_type: TypeArgument,
    pub type_parameters: Vec<TypeParameter>,
    pub where_clause: Vec<(Ident, Vec<TraitConstraint>)>,
    pub kind: FunctionDeclarationKind,
}

impl EqWithEngines for FunctionDeclaration {}
impl PartialEqWithEngines for FunctionDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.purity == other.purity
            && self.attributes == other.attributes
            && self.name == other.name
            && self.visibility == other.visibility
            && self.body.eq(&other.body, ctx)
            && self.parameters.eq(&other.parameters, ctx)
            && self.return_type.eq(&other.return_type, ctx)
            && self.type_parameters.eq(&other.type_parameters, ctx)
    }
}

impl DebugWithEngines for FunctionDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _engines: &Engines) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name))
    }
}

impl Named for FunctionDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for FunctionDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub name: Ident,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub mutability_span: Span,
    pub type_argument: TypeArgument,
}

impl EqWithEngines for FunctionParameter {}
impl PartialEqWithEngines for FunctionParameter {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.is_reference == other.is_reference
            && self.is_mutable == other.is_mutable
            && self.mutability_span == other.mutability_span
            && self.type_argument.eq(&other.type_argument, ctx)
    }
}

impl FunctionDeclaration {
    /// Checks if this `FunctionDeclaration` is a test.
    pub(crate) fn is_test(&self) -> bool {
        self.attributes
            .keys()
            .any(|k| matches!(k, AttributeKind::Test))
    }
}
