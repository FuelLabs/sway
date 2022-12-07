mod r#abstract;
mod allocated;
mod r#final;

use super::{
    register_sequencer::RegisterSequencer, AbstractInstructionSet, AllocatedAbstractInstructionSet,
    DataSection, InstructionSet,
};

use crate::{asm_lang::Label, declaration_engine::DeclarationId};

type SelectorOpt = Option<[u8; 4]>;
type FnName = String;
type ImmOffset = u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramKind {
    Contract,
    Library,
    Predicate,
    Script,
}

/// An AbstractProgram represents code generated by the compilation from IR, with virtual registers
/// and abstract control flow.
///
/// Use `AbstractProgram::to_allocated_program()` to perform register allocation.
///
pub(super) struct AbstractProgram {
    kind: ProgramKind,
    data_section: DataSection,
    entries: Vec<AbstractEntry>,
    non_entries: Vec<AbstractInstructionSet>,
    reg_seqr: RegisterSequencer,
}

/// The entry point of an abstract program.
pub(super) struct AbstractEntry {
    pub(super) selector: SelectorOpt,
    pub(super) label: Label,
    pub(super) ops: AbstractInstructionSet,
    pub(super) name: FnName,
    pub(super) test_decl_id: Option<DeclarationId>,
}

/// An AllocatedProgram represents code which has allocated registers but still has abstract
/// control flow.
pub(super) struct AllocatedProgram {
    kind: ProgramKind,
    data_section: DataSection,
    prologue: AllocatedAbstractInstructionSet,
    functions: Vec<AllocatedAbstractInstructionSet>,
    entries: Vec<(SelectorOpt, Label, FnName, Option<DeclarationId>)>,
}

/// A FinalProgram represents code which may be serialized to VM bytecode.
pub(super) struct FinalProgram {
    kind: ProgramKind,
    data_section: DataSection,
    ops: InstructionSet,
    entries: Vec<(SelectorOpt, ImmOffset, FnName, Option<DeclarationId>)>,
}
