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
    pub attributes: transform::Attributes,
    pub name: Ident,
    pub visibility: Visibility,
    pub body: CodeBlock,
    pub parameters: Vec<FunctionParameter>,
    pub span: Span,
    pub return_type: GenericArgument,
    pub type_parameters: Vec<TypeParameter>,
    pub where_clause: Vec<(Ident, Vec<TraitConstraint>)>,
    pub kind: FunctionDeclarationKind,
    pub implementing_type: Option<Declaration>,
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        f.write_fmt(format_args!("{} (", self.name))?;
        for p in self.parameters.iter() {
            f.write_fmt(format_args!("{}: ", p.name))?;
            f.write_fmt(format_args!("{}, ", engines.help_out(&p.type_argument)))?;
        }
        f.write_fmt(format_args!(") ->"))?;
        f.write_fmt(format_args!(" {:?}\n", engines.help_out(&self.return_type)))?;

        for node in self.body.contents.iter() {
            f.write_fmt(format_args!("        "))?;
            match &node.content {
                AstNodeContent::UseStatement(use_statement) => todo!(),
                AstNodeContent::Declaration(declaration) => todo!(),
                AstNodeContent::Expression(expression) => {
                    f.write_fmt(format_args!("{:?}", engines.help_out(&expression)))?;
                },
                AstNodeContent::IncludeStatement(include_statement) => todo!(),
                AstNodeContent::Error(spans, error_emitted) => todo!(),
            }
            f.write_fmt(format_args!("\n"))?;
        }

        Ok(())
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
    pub type_argument: GenericArgument,
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
    /// Checks if this [FunctionDeclaration] is a test.
    pub(crate) fn is_test(&self) -> bool {
        self.attributes.has_any_of_kind(AttributeKind::Test)
    }
}
