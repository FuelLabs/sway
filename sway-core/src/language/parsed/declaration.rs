mod abi;
mod constant;
mod r#enum;
pub mod function;
mod impl_trait;
mod storage;
mod r#struct;
mod r#trait;
mod type_alias;
mod variable;

pub use abi::*;
pub use constant::*;
pub use function::*;
pub use impl_trait::*;
pub use r#enum::*;
pub use r#struct::*;
pub use r#trait::*;
pub use storage::*;
use sway_types::Spanned;
pub use type_alias::*;
pub use variable::*;

use crate::{decl_engine::parsed_id::ParsedDeclId, Engines};

#[derive(Debug, Clone)]
pub enum Declaration {
    VariableDeclaration(ParsedDeclId<VariableDeclaration>),
    FunctionDeclaration(ParsedDeclId<FunctionDeclaration>),
    TraitDeclaration(ParsedDeclId<TraitDeclaration>),
    StructDeclaration(ParsedDeclId<StructDeclaration>),
    EnumDeclaration(ParsedDeclId<EnumDeclaration>),
    ImplTrait(ParsedDeclId<ImplTrait>),
    ImplSelf(ParsedDeclId<ImplSelf>),
    AbiDeclaration(ParsedDeclId<AbiDeclaration>),
    ConstantDeclaration(ParsedDeclId<ConstantDeclaration>),
    StorageDeclaration(ParsedDeclId<StorageDeclaration>),
    TypeAliasDeclaration(ParsedDeclId<TypeAliasDeclaration>),
    TraitTypeDeclaration(ParsedDeclId<TraitTypeDeclaration>),
}

impl Declaration {
    /// Checks if this `Declaration` is a test.
    pub(crate) fn is_test(&self, engines: &Engines) -> bool {
        if let Declaration::FunctionDeclaration(fn_decl) = self {
            let fn_decl = engines.pe().get_function(fn_decl);
            fn_decl.is_test()
        } else {
            false
        }
    }

    #[allow(dead_code)]
    fn span(&self, engines: &Engines) -> sway_types::Span {
        use Declaration::*;
        let pe = engines.pe();
        match self {
            VariableDeclaration(decl_id) => pe.get_variable(decl_id).span(),
            FunctionDeclaration(decl_id) => pe.get_function(decl_id).span(),
            TraitDeclaration(decl_id) => pe.get_trait(decl_id).span(),
            StructDeclaration(decl_id) => pe.get_struct(decl_id).span(),
            EnumDeclaration(decl_id) => pe.get_enum(decl_id).span(),
            ImplTrait(decl_id) => pe.get_impl_trait(decl_id).span(),
            ImplSelf(decl_id) => pe.get_impl_self(decl_id).span(),
            AbiDeclaration(decl_id) => pe.get_abi(decl_id).span(),
            ConstantDeclaration(decl_id) => pe.get_constant(decl_id).span(),
            StorageDeclaration(decl_id) => pe.get_storage(decl_id).span(),
            TypeAliasDeclaration(decl_id) => pe.get_type_alias(decl_id).span(),
            TraitTypeDeclaration(decl_id) => pe.get_trait_type(decl_id).span(),
        }
    }
}
