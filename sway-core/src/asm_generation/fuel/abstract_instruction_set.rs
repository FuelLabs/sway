use super::{
    allocated_abstract_instruction_set::AllocatedAbstractInstructionSet, register_allocator,
};
use crate::asm_lang::{allocated_ops::AllocatedOp, Op, RealizedOp};
use std::fmt;
use sway_error::error::CompileError;

/// An [AbstractInstructionSet] is a set of instructions that use entirely virtual registers
/// and excessive moves, with the intention of later optimizing it.
///
/// If `function` is `Some`, then this instruction set belongs to a function.
#[derive(Clone)]
pub struct AbstractInstructionSet {
    /// `Some` if this instruction set belongs to a single function.
    /// The `String` is the function name, and the `bool` is whether it's an entry function.
    pub(crate) function: Option<(String, bool)>,
    pub(crate) ops: Vec<Op>,
}

impl AbstractInstructionSet {
    /// Allocate registers.
    pub(crate) fn allocate_registers(
        self,
    ) -> Result<AllocatedAbstractInstructionSet, CompileError> {
        Ok(AllocatedAbstractInstructionSet {
            function: self.function,
            ops: register_allocator::allocate_registers(&self.ops)?,
        })
    }
}

impl fmt::Display for AbstractInstructionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".program:\n{}",
            self.ops
                .iter()
                .filter_map(|x| {
                    let line = format!("{x}");
                    if line.is_empty() {
                        None
                    } else {
                        Some(line)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// "Realized" here refers to labels -- there are no more organizational
/// ops or labels. In this struct, they are all "realized" to offsets.
pub struct RealizedAbstractInstructionSet {
    pub(super) ops: Vec<RealizedOp>,
}

impl RealizedAbstractInstructionSet {
    /// Patch the prologue `ADDI $cs, $ds, imm` (emitted by
    /// [`AllocatedAbstractInstructionSet::try_rewrite_cs_init_to_addi`]) with the
    /// final configurables-region byte offset. Must run after far-jump realization
    /// has finished inserting pointer words into the data section.
    pub(crate) fn patch_cs_addi_immediate(&mut self, configurables_offset_from_ds: u64) {
        use crate::asm_lang::{
            allocated_ops::{AllocatedInstruction, AllocatedRegister},
            ConstantRegister, VirtualImmediate12,
        };

        let cs = AllocatedRegister::Constant(ConstantRegister::ConfigurableSectionStart);
        let ds = AllocatedRegister::Constant(ConstantRegister::DataSectionStart);
        let imm = VirtualImmediate12::new(configurables_offset_from_ds);

        for op in &mut self.ops {
            if let AllocatedInstruction::ADDI(dst, src, _) = &op.opcode {
                if *dst == cs && *src == ds {
                    op.opcode = AllocatedInstruction::ADDI(cs, ds, imm.clone());
                    op.comment = "initialize $cs from $ds".into();
                    return;
                }
            }
        }
    }

    pub(crate) fn lower_to_allocated_ops(self) -> Vec<AllocatedOp> {
        self.ops
            .into_iter()
            .map(
                |RealizedOp {
                     opcode,
                     comment,
                     owning_span,
                 }| {
                    AllocatedOp {
                        opcode,
                        comment,
                        owning_span,
                    }
                },
            )
            .collect::<Vec<_>>()
    }
}
