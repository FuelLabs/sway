pub mod abi;
pub use abi::*;
pub mod asm_builder;
pub mod evm;
pub use evm::*;
pub mod from_ir;
pub mod fuel;
pub mod instruction_set;

mod finalized_asm;
pub use finalized_asm::*;
pub use fuel::data_section::{Datum, Entry};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramKind {
    Contract,
    Library,
    Predicate,
    Script,
}
