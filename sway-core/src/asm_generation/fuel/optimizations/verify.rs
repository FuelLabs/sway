use crate::{
    asm_generation::fuel::abstract_instruction_set::AbstractInstructionSet,
    asm_lang::{ConstantRegister, ControlFlowOp, VirtualRegister},
};
use std::collections::HashSet;
use either::Either;
use sway_error::error::CompileError;
use sway_types::Span;

impl AbstractInstructionSet {
    pub(crate) fn verify(self) -> Result<AbstractInstructionSet, CompileError> {
        // Check `ReturnFromCall` is correct
        for op in self.ops.iter() {
            match &op.opcode {
                Either::Right(ControlFlowOp::ReturnFromCall { zero, reta }) => {
                    if !matches!(zero, VirtualRegister::Constant(ConstantRegister::Zero)) {
                        return Err(CompileError::Internal("ReturnFromCall incorrectly not using $zero", Span::dummy()));
                    }

                    if !matches!(reta, VirtualRegister::Constant(ConstantRegister::CallReturnAddress)) {
                        return Err(CompileError::Internal("ReturnFromCall incorrectly not using $reta", Span::dummy()));
                    }
                }
                _ => {}
            }
        }

        // At the moment the only verification we do is to make sure used registers are
        // initialised.  Without doing dataflow analysis we still can't guarantee the init is
        // _before_ the use, but future refactoring to convert abstract ops into SSA and BBs will
        // make this possible or even make this check redundant.
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
