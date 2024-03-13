use super::{ConstantDeclaration, FunctionDeclaration, TraitTypeDeclaration};
use crate::{
    decl_engine::parsed_id::ParsedDeclId, engine_threading::DebugWithEngines, language::CallPath,
    type_system::TypeArgument, Engines, TypeParameter,
};

use sway_types::span::Span;

#[derive(Debug, Clone)]
pub enum ImplItem {
    Fn(ParsedDeclId<FunctionDeclaration>),
    Constant(ParsedDeclId<ConstantDeclaration>),
    Type(ParsedDeclId<TraitTypeDeclaration>),
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

#[derive(Debug, Clone)]
pub struct ImplTrait {
    pub impl_type_parameters: Vec<TypeParameter>,
    pub trait_name: CallPath,
    pub trait_type_arguments: Vec<TypeArgument>,
    pub implementing_for: TypeArgument,
    pub items: Vec<ImplItem>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
}

impl DebugWithEngines for ImplTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "impl {} for {:?}",
            self.trait_name,
            engines.help_out(self.implementing_for.clone())
        ))
    }
}

/// An impl of methods without a trait
/// like `impl MyType { fn foo { .. } }`
#[derive(Debug, Clone)]
pub struct ImplSelf {
    pub impl_type_parameters: Vec<TypeParameter>,
    pub implementing_for: TypeArgument,
    pub items: Vec<ImplItem>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
}

impl DebugWithEngines for ImplSelf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "impl {}",
            engines.help_out(self.implementing_for.clone())
        ))
    }
}
