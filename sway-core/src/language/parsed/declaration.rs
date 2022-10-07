mod abi;
mod constant;
mod r#enum;
pub mod function;
mod impl_trait;
mod storage;
mod r#struct;
mod r#trait;
mod variable;

pub(crate) use abi::*;
pub use constant::*;
pub use function::*;
pub(crate) use impl_trait::*;
pub use r#enum::*;
pub use r#struct::*;
pub use r#trait::*;
pub use storage::*;
pub use variable::*;

#[derive(Debug, Clone)]
pub enum Declaration {
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    TraitDeclaration(TraitDeclaration),
    StructDeclaration(StructDeclaration),
    EnumDeclaration(EnumDeclaration),
    ImplTrait(ImplTrait),
    ImplSelf(ImplSelf),
    AbiDeclaration(AbiDeclaration),
    ConstantDeclaration(ConstantDeclaration),
    StorageDeclaration(StorageDeclaration),
}
