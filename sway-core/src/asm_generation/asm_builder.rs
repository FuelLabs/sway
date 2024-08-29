use super::FinalizedAsm;
use crate::{asm_lang::Label, BuildConfig};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_ir::{ConfigContent, Function};

pub trait AsmBuilder {
    fn func_to_labels(&mut self, func: &Function) -> (Label, Label);
    fn compile_configurable(&mut self, config: &ConfigContent);
    fn compile_function(
        &mut self,
        handler: &Handler,
        function: Function,
    ) -> Result<(), ErrorEmitted>;
    fn finalize(
        self,
        handler: &Handler,
        build_config: Option<&BuildConfig>,
        fallback_fn: Option<Label>,
    ) -> Result<FinalizedAsm, ErrorEmitted>;
}
