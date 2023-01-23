use sway_ir::Function;

use crate::{asm_lang::Label, CompileResult};

use super::{evm::EvmAsmBuilderResult, fuel::FuelAsmBuilderResult};

pub enum AsmBuilderResult {
    Fuel(FuelAsmBuilderResult),
    Evm(EvmAsmBuilderResult),
}

pub trait AsmBuilder {
    fn func_to_labels(&mut self, func: &Function) -> (Label, Label);
    fn compile_function(&mut self, function: Function) -> CompileResult<()>;
    fn finalize(&self) -> AsmBuilderResult;
}
