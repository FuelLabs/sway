mod abi;
mod constant;
mod r#enum;
pub mod function;
mod impl_trait;
mod reassignment;
mod storage;
mod r#struct;
mod r#trait;
mod type_argument;
mod type_parameter;
mod variable;

pub(crate) use abi::*;
pub use constant::*;
pub use function::*;
pub(crate) use impl_trait::*;
pub use r#enum::*;
pub use r#struct::*;
pub use r#trait::*;
pub(crate) use reassignment::*;
pub use storage::*;
pub(crate) use type_argument::*;
pub(crate) use type_parameter::*;
pub use variable::*;



#[derive(Debug, Clone)]
pub enum Declaration {
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    TraitDeclaration(TraitDeclaration),
    StructDeclaration(StructDeclaration),
    EnumDeclaration(EnumDeclaration),
    Reassignment(Reassignment),
    ImplTrait(ImplTrait),
    ImplSelf(ImplSelf),
    AbiDeclaration(AbiDeclaration),
    ConstantDeclaration(ConstantDeclaration),
    StorageDeclaration(StorageDeclaration),
}
