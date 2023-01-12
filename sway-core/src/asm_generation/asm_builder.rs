use sway_ir::Function;

use crate::{asm_lang::Label, decl_engine::DeclId, CompileResult};

use super::{
    abstract_instruction_set::AbstractInstructionSet, register_sequencer::RegisterSequencer,
    DataSection,
};

pub type AsmBuilderResult = (
    DataSection,
    RegisterSequencer,
    Vec<(Function, Label, AbstractInstructionSet, Option<DeclId>)>,
    Vec<AbstractInstructionSet>,
);

pub trait AsmBuilder {
    fn func_to_labels(&mut self, func: &Function) -> (Label, Label);
    fn compile_function(&mut self, function: Function) -> CompileResult<()>;
    fn finalize(&self) -> AsmBuilderResult;
}
