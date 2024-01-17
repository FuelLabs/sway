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
    TraitTypeDeclaration(TraitTypeDeclaration),
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
}
