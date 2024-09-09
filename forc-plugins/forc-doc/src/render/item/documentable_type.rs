use crate::render::title::{BlockTitle, DocBlock};
use sway_core::{language::ty::TyDecl, TypeInfo};

/// The compiler type that can be documented.
#[derive(Clone, Debug)]
pub enum DocumentableType {
    /// Any type that is declared in the Sway source code can be documented.
    Declared(TyDecl),
    /// Primitive types are not declared in the Sway source code, so they must be documented
    /// without a declaration.
    Primitive(TypeInfo),
}

impl DocumentableType {
    pub fn doc_name(&self) -> &str {
        match self {
            DocumentableType::Declared(decl) => decl.name(),
            DocumentableType::Primitive(ty) => ty.name(),
        }
    }

    pub fn as_block_title(&self) -> BlockTitle {
        match self {
            DocumentableType::Declared(decl) => decl.title(),
            DocumentableType::Primitive(ty) => ty.title(),
        }
    }

    pub fn friendly_type_name(&self) -> &str {
        match self {
            DocumentableType::Declared(decl) => decl.friendly_type_name(),
            DocumentableType::Primitive(_) => "primitive",
        }
    }
}
