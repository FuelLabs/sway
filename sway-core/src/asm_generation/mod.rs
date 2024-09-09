pub mod abi;
pub use abi::*;
pub mod asm_builder;
pub mod evm;
pub use evm::*;
mod miden_vm;
pub use miden_vm::*;
pub mod from_ir;
pub mod fuel;
pub mod instruction_set;

mod finalized_asm;
pub use finalized_asm::{CompiledBytecode, FinalizedAsm, FinalizedEntry};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramKind {
    Contract,
    Library,
    Predicate,
    Script,
}
