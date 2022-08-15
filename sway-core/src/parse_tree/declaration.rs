mod abi;
mod constant;
mod r#enum;
pub mod function;
mod impl_trait;
mod reassignment;
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
pub use reassignment::*;
pub use storage::*;
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
    Break { span: sway_types::Span },
    Continue { span: sway_types::Span },
}
