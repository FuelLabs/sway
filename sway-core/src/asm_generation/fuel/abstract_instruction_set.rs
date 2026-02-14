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
                .map(|x| format!("{x}"))
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
    pub(crate) fn allocated_ops(self) -> Vec<AllocatedOp> {
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
