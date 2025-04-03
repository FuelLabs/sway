use super::{ConstantDeclaration, FunctionDeclaration, TraitTypeDeclaration};
use crate::{
    decl_engine::{parsed_id::ParsedDeclId, ParsedInterfaceDeclId},
    engine_threading::{
        DebugWithEngines, EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext,
    },
    language::CallPath,
    type_system::GenericArgument,
    Engines, TypeParameter,
};

use sway_types::{span::Span, Named, Spanned};

#[derive(Debug, Clone)]
pub enum ImplItem {
    Fn(ParsedDeclId<FunctionDeclaration>),
    Constant(ParsedDeclId<ConstantDeclaration>),
    Type(ParsedDeclId<TraitTypeDeclaration>),
}

impl ImplItem {
    pub fn span(&self, engines: &Engines) -> Span {
        match self {
            ImplItem::Fn(id) => engines.pe().get_function(id).span(),
            ImplItem::Constant(id) => engines.pe().get_constant(id).span(),
            ImplItem::Type(id) => engines.pe().get_trait_type(id).span(),
        }
    }
}

impl EqWithEngines for ImplItem {}
impl PartialEqWithEngines for ImplItem {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (ImplItem::Fn(lhs), ImplItem::Fn(rhs)) => PartialEqWithEngines::eq(lhs, rhs, ctx),
            (ImplItem::Constant(lhs), ImplItem::Constant(rhs)) => {
                PartialEqWithEngines::eq(lhs, rhs, ctx)
            }
            (ImplItem::Type(lhs), ImplItem::Type(rhs)) => PartialEqWithEngines::eq(lhs, rhs, ctx),
            _ => false,
        }
    }
}

impl DebugWithEngines for ImplItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        match self {
            ImplItem::Fn(decl_id) => {
                let decl = engines.pe().get_function(decl_id);
                f.write_fmt(format_args!("{:?}", engines.help_out(decl)))
            }
            ImplItem::Constant(decl_id) => {
                let decl = engines.pe().get_constant(decl_id);
                f.write_fmt(format_args!("{:?}", engines.help_out(decl)))
            }
            ImplItem::Type(decl_id) => {
                let decl = engines.pe().get_trait_type(decl_id);
                f.write_fmt(format_args!("{:?}", engines.help_out(decl)))
            }
        }
    }
}

/// An impl trait, or impl self of methods without a trait.
/// like `impl MyType { fn foo { .. } }`
#[derive(Debug, Clone)]
pub struct ImplSelfOrTrait {
    pub is_self: bool,
    pub impl_type_parameters: Vec<TypeParameter>,
    pub trait_name: CallPath,
    pub trait_type_arguments: Vec<GenericArgument>,
    pub trait_decl_ref: Option<ParsedInterfaceDeclId>,
    pub implementing_for: GenericArgument,
    pub items: Vec<ImplItem>,
    /// The [Span] of the whole impl trait and block.
    pub(crate) block_span: Span,
}

impl EqWithEngines for ImplSelfOrTrait {}
impl PartialEqWithEngines for ImplSelfOrTrait {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.impl_type_parameters
            .eq(&other.impl_type_parameters, ctx)
            && self.trait_name == other.trait_name
            && self
                .trait_type_arguments
                .eq(&other.trait_type_arguments, ctx)
            && self.implementing_for.eq(&other.implementing_for, ctx)
            && self.items.eq(&other.items, ctx)
            && self.block_span == other.block_span
    }
}

impl Named for ImplSelfOrTrait {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.trait_name.suffix
    }
}

impl Spanned for ImplSelfOrTrait {
    fn span(&self) -> sway_types::Span {
        self.block_span.clone()
    }
}

impl DebugWithEngines for ImplSelfOrTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        if self.is_self {
            f.write_fmt(format_args!(
                "impl {}",
                engines.help_out(self.implementing_for.clone())
            ))
        } else {
            f.write_fmt(format_args!(
                "impl {} for {:?}",
                self.trait_name,
                engines.help_out(self.implementing_for.clone())
            ))
        }
    }
}
