use sway_error::handler::{ErrorEmitted, Handler};
use sway_ir::Function;

use crate::asm_lang::Label;

use super::{
    evm::EvmAsmBuilderResult, fuel::fuel_asm_builder::FuelAsmBuilderResult,
    miden_vm::MidenVMAsmBuilderResult,
};

pub enum AsmBuilderResult {
    Fuel(FuelAsmBuilderResult),
    Evm(EvmAsmBuilderResult),
    MidenVM(MidenVMAsmBuilderResult),
}

pub trait AsmBuilder {
    fn func_to_labels(&mut self, func: &Function) -> (Label, Label);
    fn compile_function(
        &mut self,
        handler: &Handler,
        function: Function,
    ) -> Result<(), ErrorEmitted>;
    fn finalize(&self) -> AsmBuilderResult;
}
