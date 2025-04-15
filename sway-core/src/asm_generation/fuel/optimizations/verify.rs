use std::collections::HashSet;

use sway_error::error::CompileError;
use sway_types::Span;

use crate::{
    asm_generation::fuel::abstract_instruction_set::AbstractInstructionSet,
    asm_lang::VirtualRegister,
};

impl AbstractInstructionSet {
    // At the moment the only verification we do is to make sure used registers are
    // initialised.  Without doing dataflow analysis we still can't guarantee the init is
    // _before_ the use, but future refactoring to convert abstract ops into SSA and BBs will
    // make this possible or even make this check redundant.
    pub(crate) fn verify(self) -> Result<AbstractInstructionSet, CompileError> {
        macro_rules! add_virt_regs {
            ($regs: expr, $set: expr) => {
                let mut regs = $regs;
                regs.retain(|&reg| matches!(reg, VirtualRegister::Virtual(_)));
                $set.extend(regs.into_iter());
            };
        }

        let mut use_regs = HashSet::new();
        let mut def_regs = HashSet::new();
        for op in &self.ops {
            add_virt_regs!(op.use_registers(), use_regs);
            add_virt_regs!(op.def_registers(), def_regs);
        }

        if def_regs.is_superset(&use_regs) {
            Ok(self)
        } else {
            let bad_regs = use_regs
                .difference(&def_regs)
                .map(|reg| match reg {
                    VirtualRegister::Virtual(name) => format!("$r{name}"),
                    VirtualRegister::Constant(creg) => creg.to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            Err(CompileError::InternalOwned(
                format!("Program erroneously uses uninitialized virtual registers: {bad_regs}"),
                Span::dummy(),
            ))
        }
    }
}
