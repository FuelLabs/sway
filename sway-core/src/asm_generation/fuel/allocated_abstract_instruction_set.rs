use crate::{
    asm_generation::fuel::data_section::EntryName,
    asm_lang::{
        allocated_ops::{AllocatedInstruction, AllocatedRegister},
        AllocatedAbstractOp, ConstantRegister, ControlFlowOp, JumpType, Label, RealizedOp,
        VirtualImmediate12, VirtualImmediate18, VirtualImmediate24,
    },
};

use super::{
    abstract_instruction_set::RealizedAbstractInstructionSet,
    compiler_constants as consts,
    data_section::{DataSection, Entry},
};

use fuel_vm::prelude::Instruction;
use indexmap::{IndexMap, IndexSet};
use rustc_hash::FxHashMap;
use sway_types::span::Span;

use std::collections::{BTreeSet, HashMap};

use either::Either;

// Convenience type for representing a map from a label to its offset and number of instructions
// following it until the next label (i.e., the length of the basic block).
pub(crate) type LabeledBlocks = HashMap<Label, BasicBlock>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct BasicBlock {
    pub(crate) offs: u64,
}

#[derive(Clone)]
pub struct AllocatedAbstractInstructionSet {
    pub(crate) ops: Vec<AllocatedAbstractOp>,
}

impl AllocatedAbstractInstructionSet {
    /// Optimize the instruction set **of a single function**.
    ///
    /// `is_entry` is true if the function is an entry point of the program.
    pub(crate) fn optimize(self, is_entry: bool) -> AllocatedAbstractInstructionSet {
        self.remove_redundant_sp_move_to_locbase(is_entry)
            .remove_redundant_ops()
    }

    /// Remove the `MOVE $$locbase, $$sp` instruction in non-entry functions, if the function does
    /// not use stack at all. This means the function:
    ///  - does not have any locals
    ///  - does not call any functions that spill arguments to the stack
    ///  - does not spill any registers to the stack when allocating registers
    ///
    /// This is the cases IFF the function contains `CFEI 0`.
    fn remove_redundant_sp_move_to_locbase(
        mut self,
        is_entry: bool,
    ) -> AllocatedAbstractInstructionSet {
        if is_entry {
            // We need to keep the $$sp move to $$locbase in entry functions, as it is a part of the compiler's contract.
            // E.g., when calculating jump instruction index, `forc test` relies on the fact that
            // every test entry function has `MOVE $$locbase, $$sp`.
            return self;
        }

        let has_zero_cfei = self.ops.iter().any(|op| matches!(&op.opcode, Either::Left(AllocatedInstruction::CFEI(imm)) if imm.value() == 0));

        if !has_zero_cfei {
            return self;
        }

        // If there is a `CFEI 0` then we know for sure that the function does not use stack at all.
        // This means $$locbase is not used anywhere and we can safely remove the `MOVE $$locbase, $$sp` instruction.
        // If $$locbase is used, this is a compilation error and will be caught by the verifier later on.
        if let Some(move_to_locals_base_idx) = self.ops.iter().position(|op| {
            matches!(
                &op.opcode,
                Either::Left(AllocatedInstruction::MOVE(
                    AllocatedRegister::Constant(ConstantRegister::LocalsBase),
                    AllocatedRegister::Constant(ConstantRegister::StackPointer),
                ))
            )
        }) {
            self.ops.remove(move_to_locals_base_idx);
        }

        self
    }

    fn remove_redundant_ops(mut self) -> AllocatedAbstractInstructionSet {
        self.ops.retain(|op| {
            // It is easier to think in terms of operations we want to remove
            // than the operations we want to retain ;-)
            let remove = match &op.opcode {
                // `cfei i0` and `cfsi i0` pairs.
                Either::Left(AllocatedInstruction::CFEI(imm))
                | Either::Left(AllocatedInstruction::CFSI(imm)) => imm.value() == 0u32,
                // `cfe $zero` and `cfs $zero` pairs.
                Either::Left(AllocatedInstruction::CFE(reg))
                | Either::Left(AllocatedInstruction::CFS(reg)) => reg.is_zero(),
                _ => false,
            };

            !remove
        });

        self
    }

    /// Replace each PUSHA instruction with pushing of all used registers to the stack, and each
    /// POPA with respective popping from the stack.
    ///
    /// Effectively, we are lowering the abstract PUSHA/POPA instructions into PSHL/PSHH and POPL/POPH instructions.
    ///
    /// Typically there will be only one of each but the code here allows for nested sections or
    /// even overlapping sections.
    pub(crate) fn emit_pusha_popa(mut self) -> Self {
        // Gather the sets of used registers per section.  Using a fold here because it's actually
        // simpler to manage.  We use a HashSet to keep track of the active section labels and then
        // build a HashMap of Label to HashSet of registers.
        let reg_sets = self
            .ops
            .iter()
            .fold(
                (IndexMap::new(), IndexSet::new()),
                |(mut reg_sets, mut active_sets), op| {
                    let regs: Box<dyn Iterator<Item = &AllocatedRegister>> = match &op.opcode {
                        Either::Right(ControlFlowOp::PushAll(label)) => {
                            active_sets.insert(*label);
                            Box::new(std::iter::empty())
                        }
                        Either::Right(ControlFlowOp::PopAll(label)) => {
                            active_sets.swap_remove(label);
                            Box::new(std::iter::empty())
                        }

                        Either::Left(alloc_op) => Box::new(alloc_op.def_registers().into_iter()),
                        Either::Right(ctrl_op) => Box::new(ctrl_op.def_registers().into_iter()),
                    };

                    for reg in regs {
                        for active_label in active_sets.clone() {
                            reg_sets
                                .entry(active_label)
                                .and_modify(|regs: &mut BTreeSet<AllocatedRegister>| {
                                    regs.insert(reg.clone());
                                })
                                .or_insert_with(|| {
                                    BTreeSet::from_iter(std::iter::once(reg).cloned())
                                });
                        }
                    }

                    (reg_sets, active_sets)
                },
            )
            .0;

        fn generate_mask(regs: &[&AllocatedRegister]) -> (VirtualImmediate24, VirtualImmediate24) {
            let mask = regs.iter().fold((0, 0), |mut accum, reg| {
                let reg_id = reg.to_reg_id().to_u8();
                assert!((16..64).contains(&reg_id));
                let reg_id = reg_id - 16;
                let (mask_ref, bit) = if reg_id < 24 {
                    (&mut accum.0, reg_id)
                } else {
                    (&mut accum.1, reg_id - 24)
                };
                // Set bit (from the least significant side) of mask_ref.
                *mask_ref |= 1 << bit;
                accum
            });
            (
                VirtualImmediate24::try_new(mask.0, Span::dummy())
                    .expect("mask should have fit in 24b"),
                VirtualImmediate24::try_new(mask.1, Span::dummy())
                    .expect("mask should have fit in 24b"),
            )
        }

        // Now replace the PUSHA/POPA instructions with PSHL/PSHH and POPL/POPH instructions.
        self.ops = self.ops.drain(..).fold(Vec::new(), |mut new_ops, op| {
            match &op.opcode {
                Either::Right(ControlFlowOp::PushAll(label)) => {
                    let regs = reg_sets
                        .get(label)
                        .expect("Have collected registers above.")
                        .iter()
                        .filter(|reg| matches!(reg, AllocatedRegister::Allocated(_)))
                        .chain([&AllocatedRegister::Constant(ConstantRegister::LocalsBase)])
                        .collect::<Vec<_>>();

                    let (mask_l, mask_h) = generate_mask(&regs);
                    if mask_l.value() != 0 {
                        new_ops.push(AllocatedAbstractOp {
                            opcode: Either::Left(AllocatedInstruction::PSHL(mask_l)),
                            comment: "push used low registers 16..40 to the stack".into(),
                            owning_span: op.owning_span.clone(),
                        });
                    }
                    if mask_h.value() != 0 {
                        new_ops.push(AllocatedAbstractOp {
                            opcode: Either::Left(AllocatedInstruction::PSHH(mask_h)),
                            comment: "push used high registers 40..64 to the stack".into(),
                            owning_span: op.owning_span.clone(),
                        });
                    }
                }

                Either::Right(ControlFlowOp::PopAll(label)) => {
                    let regs = reg_sets
                        .get(label)
                        .expect("Have collected registers above.")
                        .iter()
                        .filter(|reg| matches!(reg, AllocatedRegister::Allocated(_)))
                        .chain([&AllocatedRegister::Constant(ConstantRegister::LocalsBase)])
                        .collect::<Vec<_>>();

                    let (mask_l, mask_h) = generate_mask(&regs);
                    if mask_h.value() != 0 {
                        new_ops.push(AllocatedAbstractOp {
                            opcode: Either::Left(AllocatedInstruction::POPH(mask_h)),
                            comment: "restore used high registers 40..64 from the stack".into(),
                            owning_span: op.owning_span.clone(),
                        });
                    }
                    if mask_l.value() != 0 {
                        new_ops.push(AllocatedAbstractOp {
                            opcode: Either::Left(AllocatedInstruction::POPL(mask_l)),
                            comment: "restore used low registers 16..40 from the stack".into(),
                            owning_span: op.owning_span.clone(),
                        });
                    }
                }

                _otherwise => new_ops.push(op),
            };
            new_ops
        });

        self
    }

    /// Runs two passes -- one to get the instruction offsets of the labels and one to replace the
    /// labels in the organizational ops
    pub(crate) fn realize_labels(
        mut self,
        data_section: &mut DataSection,
        far_jump_sizes: &FxHashMap<usize, u64>,
    ) -> Result<(RealizedAbstractInstructionSet, LabeledBlocks), crate::CompileError> {
        let label_offsets = self.resolve_labels(data_section);
        let mut curr_offset = 0;

        let mut realized_ops = vec![];
        for (op_idx, op) in self.ops.iter().enumerate() {
            let op_size = far_jump_sizes
                .get(&op_idx)
                .copied()
                .unwrap_or_else(|| Self::instruction_size_not_far_jump(op, data_section));
            let AllocatedAbstractOp {
                opcode,
                comment,
                owning_span,
            } = op.clone();
            match opcode {
                Either::Left(op) => realized_ops.push(RealizedOp {
                    opcode: op,
                    owning_span,
                    comment,
                }),
                Either::Right(org_op) => match org_op {
                    ControlFlowOp::Jump { to, type_ } => {
                        let target_offset = label_offsets.get(&to).unwrap().offs;
                        let ops = if matches!(type_, JumpType::Call) {
                            compile_call(
                                data_section,
                                curr_offset,
                                target_offset,
                                far_jump_sizes.get(&op_idx).copied(),
                                comment,
                                owning_span,
                            )
                        } else {
                            compile_jump(
                                data_section,
                                curr_offset,
                                target_offset,
                                match type_ {
                                    JumpType::NotZero(cond) => Some(cond),
                                    _ => None,
                                },
                                far_jump_sizes.contains_key(&op_idx),
                                comment,
                                owning_span,
                            )
                        };
                        debug_assert_eq!(ops.len() as u64, op_size);
                        realized_ops.extend(ops);
                    }
                    ControlFlowOp::DataSectionOffsetPlaceholder => {
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedInstruction::DataSectionOffsetPlaceholder,
                            owning_span: None,
                            comment: String::new(),
                        });
                    }
                    ControlFlowOp::ConfigurablesOffsetPlaceholder => {
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedInstruction::ConfigurablesOffsetPlaceholder,
                            owning_span: None,
                            comment: String::new(),
                        });
                    }
                    ControlFlowOp::Comment => continue,
                    ControlFlowOp::Label(..) => continue,

                    ControlFlowOp::PushAll(_) | ControlFlowOp::PopAll(_) => {
                        unreachable!("still don't belong in organisational ops")
                    }
                },
            };
            curr_offset += op_size;
        }

        let realized_ops = RealizedAbstractInstructionSet { ops: realized_ops };
        Ok((realized_ops, label_offsets))
    }

    /// Resolve jump label offsets.
    ///
    /// For very large programs the label offsets may be too large to fit in an immediate part
    /// of the jump instruction. In these case we must use a register value as a jump target.
    /// This requires two instructions, one to load the destination register and then the jump itself.
    ///
    /// But we don't know the offset of a label until we scan through the ops and count them.
    /// So we have a chicken and egg situation where we may need to add new instructions which
    /// would change the offsets to all labels thereafter, which in turn could require more
    /// instructions to be added, and so on.
    ///
    /// For this reason, we take a two-pass approach. On the first pass, we pessimistically assume
    /// that all jumps may require take two opcodes, and use this assumption to calculate the
    /// offsets of labels. Then we see which jumps actually require two opcodes and mark them as such.
    /// This approach is not optimal as it sometimes requires more opcodes than necessary,
    /// but it is simple and quite works well in practice.
    fn resolve_labels(&mut self, data_section: &mut DataSection) -> LabeledBlocks {
        let far_jump_indices = self.collect_far_jumps();
        self.map_label_offsets(data_section, &far_jump_indices)
    }

    // Returns largest size an instruction can take up.
    // The return value is in concrete instructions, i.e. units of 4 bytes.
    fn worst_case_instruction_size(op: &AllocatedAbstractOp) -> u64 {
        use ControlFlowOp::*;
        match op.opcode {
            Either::Right(Label(_)) => 0,

            // Loads from data section may take up to 2 instructions
            Either::Left(
                AllocatedInstruction::LoadDataId(_, _) | AllocatedInstruction::AddrDataId(_, _),
            ) => 2,

            // cfei 0 and cfsi 0 are omitted from asm emission, don't count them for offsets
            Either::Left(AllocatedInstruction::CFEI(ref op))
            | Either::Left(AllocatedInstruction::CFSI(ref op))
                if op.value() == 0 =>
            {
                0
            }

            // Another special case for the blob opcode, used for testing.
            Either::Left(AllocatedInstruction::BLOB(ref count)) => count.value() as u64,

            // This is a concrete op, size is fixed
            Either::Left(_) => 1,

            // Worst case for jump is 2 opcodes, and 3 for calls
            Either::Right(Jump { ref type_, .. }) => match type_ {
                JumpType::Unconditional => 2,
                JumpType::NotZero(_) => 2,
                JumpType::Call => 3,
            },

            Either::Right(Comment) => 0,

            Either::Right(DataSectionOffsetPlaceholder) => {
                // If the placeholder is 32 bits, this is 1. if 64, this should be 2. We use LW
                // to load the data, which loads a whole word, so for now this is 2.
                2
            }

            Either::Right(ConfigurablesOffsetPlaceholder) => 2,

            Either::Right(PushAll(_)) | Either::Right(PopAll(_)) => unreachable!(
                "fix me, pushall and popall don't really belong in control flow ops \
                        since they're not about control flow"
            ),
        }
    }

    // Actual size of an instruction.
    // Note that this return incorrect values for far jumps, they must be handled separately.
    // The return value is in concrete instructions, i.e. units of 4 bytes.
    fn instruction_size_not_far_jump(op: &AllocatedAbstractOp, data_section: &DataSection) -> u64 {
        use ControlFlowOp::*;
        match op.opcode {
            Either::Right(Label(_)) => 0,

            // A special case for LoadDataId which may be 1 or 2 ops, depending on the source size.
            Either::Left(AllocatedInstruction::LoadDataId(_, ref data_id)) => {
                let has_copy_type = data_section.has_copy_type(data_id).expect(
                    "Internal miscalculation in data section -- \
                        data id did not match up to any actual data",
                );
                if has_copy_type {
                    1
                } else {
                    2
                }
            }

            Either::Left(AllocatedInstruction::AddrDataId(_, ref _data_id)) => 2,

            // cfei 0 and cfsi 0 are omitted from asm emission, don't count them for offsets
            Either::Left(AllocatedInstruction::CFEI(ref op))
            | Either::Left(AllocatedInstruction::CFSI(ref op))
                if op.value() == 0 =>
            {
                0
            }

            // Another special case for the blob opcode, used for testing.
            Either::Left(AllocatedInstruction::BLOB(ref count)) => count.value() as u64,

            // This is a concrete op, size is fixed
            Either::Left(_) => 1,

            // Far jumps must be handled separately, as they require two instructions.
            Either::Right(Jump { .. }) => 1,

            Either::Right(Comment) => 0,

            Either::Right(DataSectionOffsetPlaceholder) => {
                // If the placeholder is 32 bits, this is 1. if 64, this should be 2. We use LW
                // to load the data, which loads a whole word, so for now this is 2.
                2
            }

            Either::Right(ConfigurablesOffsetPlaceholder) => 2,

            Either::Right(PushAll(_)) | Either::Right(PopAll(_)) => unreachable!(
                "fix me, pushall and popall don't really belong in control flow ops \
                        since they're not about control flow"
            ),
        }
    }

    /// Go through all jumps and check if they could require a far jump in the worst case.
    /// For far jumps we have to reserve space for an extra opcode to load target address.
    /// For far calls, we need to reserve two extra opcodes.
    /// Also, this will be mark self-jumps, as they require a noop to be inserted before them.
    pub(crate) fn collect_far_jumps(&self) -> FxHashMap<usize, u64> {
        let mut labelled_blocks = LabeledBlocks::new();
        let mut cur_offset = 0;
        let mut cur_basic_block = None;

        let mut far_jump_sizes = FxHashMap::default();

        struct JumpInfo {
            to: Label,
            offset: u64,
            op_idx: usize,
        }

        let mut jumps = Vec::new();

        for (op_idx, op) in self.ops.iter().enumerate() {
            // If we're seeing a control flow op then it's the end of the block.
            if let Either::Right(ControlFlowOp::Label(_) | ControlFlowOp::Jump { .. }) = op.opcode {
                if let Some((lab, _idx, offs)) = cur_basic_block {
                    // Insert the previous basic block.
                    labelled_blocks.insert(lab, BasicBlock { offs });
                }
            }
            if let Either::Right(ControlFlowOp::Label(cur_lab)) = op.opcode {
                // Save the new block label and furthest offset.
                cur_basic_block = Some((cur_lab, op_idx, cur_offset));
            }

            if let Either::Right(ControlFlowOp::Jump { to, .. }) = &op.opcode {
                jumps.push(JumpInfo {
                    to: *to,
                    offset: cur_offset,
                    op_idx,
                });
            }

            // Update the offset.
            cur_offset += Self::worst_case_instruction_size(op);
        }

        // Don't forget the final block.
        if let Some((lab, _idx, offs)) = cur_basic_block {
            labelled_blocks.insert(lab, BasicBlock { offs });
        }

        for jump in jumps {
            let offs = labelled_blocks.get(&jump.to).unwrap().offs;
            let rel_offset = offs.abs_diff(jump.offset);
            let Either::Right(ControlFlowOp::Jump { ref type_, .. }) = self.ops[jump.op_idx].opcode
            else {
                unreachable!("Jump info should only be collected for jumps");
            };
            // Relative self jumps need a NOOP inserted before it so that we can jump to the NOOP.
            let is_self_jump = rel_offset == 0;
            match type_ {
                JumpType::Unconditional => {
                    // Unconditional jumps have 18-bit immidate offset
                    if is_self_jump || rel_offset > consts::EIGHTEEN_BITS {
                        far_jump_sizes.insert(jump.op_idx, 2);
                    }
                }
                JumpType::NotZero(_) => {
                    // Conditional jumps have 12-bit immidate offset
                    if is_self_jump || rel_offset > consts::TWELVE_BITS {
                        far_jump_sizes.insert(jump.op_idx, 2);
                    }
                }
                JumpType::Call => {
                    // Use the actual codegen to estimate the size of the call.
                    // This can never generate a number that's too small, but in some
                    // corner cases it leads to reserving an extra opcode.
                    // See `compile_call` that inserts NOOPs to pad the call in these cases.
                    let len = compile_call_inner(
                        &mut DataSection::default(),
                        jump.offset,
                        offs,
                        String::new(),
                        None,
                    )
                    .len();
                    far_jump_sizes.insert(jump.op_idx, len as u64);
                }
            };
        }

        far_jump_sizes
    }

    /// Map the labels to their offsets in the program.
    fn map_label_offsets(
        &self,
        data_section: &DataSection,
        far_jump_sizes: &FxHashMap<usize, u64>,
    ) -> LabeledBlocks {
        let mut labelled_blocks = LabeledBlocks::new();
        let mut cur_offset = 0;
        let mut cur_basic_block = None;

        for (op_idx, op) in self.ops.iter().enumerate() {
            // If we're seeing a control flow op then it's the end of the block.
            if let Either::Right(ControlFlowOp::Label(_) | ControlFlowOp::Jump { .. }) = op.opcode {
                if let Some((lab, _idx, offs)) = cur_basic_block {
                    // Insert the previous basic block.
                    labelled_blocks.insert(lab, BasicBlock { offs });
                }
            }
            if let Either::Right(ControlFlowOp::Label(cur_lab)) = op.opcode {
                // Save the new block label and furthest offset.
                cur_basic_block = Some((cur_lab, op_idx, cur_offset));
            }

            // Update the offset.
            let op_size = far_jump_sizes
                .get(&op_idx)
                .copied()
                .unwrap_or_else(|| Self::instruction_size_not_far_jump(op, data_section));
            cur_offset += op_size;
        }

        // Don't forget the final block.
        if let Some((lab, _idx, offs)) = cur_basic_block {
            labelled_blocks.insert(lab, BasicBlock { offs });
        }

        labelled_blocks
    }
}

impl std::fmt::Display for AllocatedAbstractInstructionSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            ".program:\n{}",
            self.ops
                .iter()
                .map(|op| format!("{op}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// Compiles jump into the appropriate operations.
/// Near jumps are compiled into a single instruction, while far jumps are compiled into
/// two instructions: one to load the target address and another to jump to it.
pub(crate) fn compile_jump(
    data_section: &mut DataSection,
    curr_offset: u64,
    target_offset: u64,
    condition_nonzero: Option<AllocatedRegister>,
    far: bool,
    comment: String,
    owning_span: Option<Span>,
) -> Vec<RealizedOp> {
    if curr_offset == target_offset {
        if !far {
            unreachable!("Self jump should have been marked by mark_far_jumps");
        }

        return vec![
            RealizedOp {
                opcode: AllocatedInstruction::NOOP,
                owning_span: owning_span.clone(),
                comment: "".into(),
            },
            if let Some(cond_nz) = condition_nonzero {
                RealizedOp {
                    opcode: AllocatedInstruction::JNZB(
                        cond_nz,
                        AllocatedRegister::Constant(ConstantRegister::Zero),
                        VirtualImmediate12::new(0),
                    ),
                    owning_span,
                    comment,
                }
            } else {
                RealizedOp {
                    opcode: AllocatedInstruction::JMPB(
                        AllocatedRegister::Constant(ConstantRegister::Zero),
                        VirtualImmediate18::new(0),
                    ),
                    owning_span,
                    comment,
                }
            },
        ];
    }

    if curr_offset > target_offset {
        let delta = curr_offset - target_offset - 1;
        return if far {
            let data_id = data_section.insert_data_value(Entry::new_word(
                delta + 1, // +1 since the load instruction must be skipped as well
                EntryName::NonConfigurable,
                None,
            ));

            vec![
                RealizedOp {
                    opcode: AllocatedInstruction::LoadDataId(
                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                        data_id,
                    ),
                    owning_span: owning_span.clone(),
                    comment: "load far jump target address".into(),
                },
                RealizedOp {
                    opcode: if let Some(cond_nz) = condition_nonzero {
                        AllocatedInstruction::JNZB(
                            cond_nz,
                            AllocatedRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate12::new(0),
                        )
                    } else {
                        AllocatedInstruction::JMPB(
                            AllocatedRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate18::new(0),
                        )
                    },
                    owning_span,
                    comment,
                },
            ]
        } else {
            vec![RealizedOp {
                opcode: if let Some(cond_nz) = condition_nonzero {
                    AllocatedInstruction::JNZB(
                        cond_nz,
                        AllocatedRegister::Constant(ConstantRegister::Zero),
                        VirtualImmediate12::new(delta),
                    )
                } else {
                    AllocatedInstruction::JMPB(
                        AllocatedRegister::Constant(ConstantRegister::Zero),
                        VirtualImmediate18::new(delta),
                    )
                },
                owning_span,
                comment,
            }]
        };
    }

    let delta = target_offset - curr_offset - 1;

    if far {
        let data_id = data_section.insert_data_value(Entry::new_word(
            delta - 1,
            EntryName::NonConfigurable,
            None,
        ));

        vec![
            RealizedOp {
                opcode: AllocatedInstruction::LoadDataId(
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                    data_id,
                ),
                owning_span: owning_span.clone(),
                comment: "load far jump target address".into(),
            },
            RealizedOp {
                opcode: if let Some(cond_nz) = condition_nonzero {
                    AllocatedInstruction::JNZF(
                        cond_nz,
                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                        VirtualImmediate12::new(0),
                    )
                } else {
                    AllocatedInstruction::JMPF(
                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                        VirtualImmediate18::new(0),
                    )
                },
                owning_span,
                comment,
            },
        ]
    } else {
        vec![RealizedOp {
            opcode: if let Some(cond_nz) = condition_nonzero {
                AllocatedInstruction::JNZF(
                    cond_nz,
                    AllocatedRegister::Constant(ConstantRegister::Zero),
                    VirtualImmediate12::new(delta),
                )
            } else {
                AllocatedInstruction::JMPF(
                    AllocatedRegister::Constant(ConstantRegister::Zero),
                    VirtualImmediate18::new(delta),
                )
            },
            owning_span,
            comment,
        }]
    }
}

/// Compiles a function call into the appropriate operations.
/// Generates 1 to 3 instruction depending on the distance to target.
pub(crate) fn compile_call_inner(
    data_section: &mut DataSection,
    curr_offset: u64,
    target_offset: u64,
    comment: String,
    owning_span: Option<Span>,
) -> Vec<RealizedOp> {
    // Handle forwards and backwards jumps separately
    if curr_offset <= target_offset {
        let delta = target_offset - curr_offset;

        // If the offset is small enough for a single instruction, do it directly
        if let Ok(imm) = VirtualImmediate12::try_new(delta, Span::dummy()) {
            return vec![RealizedOp {
                opcode: AllocatedInstruction::JAL(
                    AllocatedRegister::Constant(ConstantRegister::CallReturnAddress),
                    AllocatedRegister::Constant(ConstantRegister::ProgramCounter),
                    imm,
                ),
                owning_span,
                comment,
            }];
        }

        // The next approaches require an extra instruction before the PC is used, so we
        // subtract 1 from the delta to account for that. It cannot underflow as otherwise the first
        // approach would have been used. Then we multiply by instruction size for doing arithmetic
        // with the PC register. The overflow cannot occur since programs cannot be 2**60 bytes large.
        let delta_instr = (delta - 1) * (Instruction::SIZE as u64);

        // Attempt MOVI-based approach, that has larger immediate size but doesn't require data section.
        if let Ok(imm) = VirtualImmediate18::try_new(delta_instr, Span::dummy()) {
            return vec![
                RealizedOp {
                    opcode: AllocatedInstruction::MOVI(
                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                        imm,
                    ),
                    owning_span: owning_span.clone(),
                    comment: "load call target address".into(),
                },
                RealizedOp {
                    opcode: AllocatedInstruction::ADD(
                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                        AllocatedRegister::Constant(ConstantRegister::ProgramCounter),
                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                    ),
                    owning_span: owning_span.clone(),
                    comment: "load call target address".into(),
                },
                RealizedOp {
                    opcode: AllocatedInstruction::JAL(
                        AllocatedRegister::Constant(ConstantRegister::CallReturnAddress),
                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                        VirtualImmediate12::new(0),
                    ),
                    owning_span,
                    comment,
                },
            ];
        }

        // if the offset is too large for MOVI, use data section to store the full offset.
        let data_id = data_section.insert_data_value(Entry::new_word(
            delta_instr,
            EntryName::NonConfigurable,
            None,
        ));

        return vec![
            RealizedOp {
                opcode: AllocatedInstruction::LoadDataId(
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                    data_id,
                ),
                owning_span: owning_span.clone(),
                comment: "load call target address".into(),
            },
            RealizedOp {
                opcode: AllocatedInstruction::ADD(
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                    AllocatedRegister::Constant(ConstantRegister::ProgramCounter),
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                ),
                owning_span: owning_span.clone(),
                comment: "load call target address".into(),
            },
            RealizedOp {
                opcode: AllocatedInstruction::JAL(
                    AllocatedRegister::Constant(ConstantRegister::CallReturnAddress),
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                    VirtualImmediate12::new(0),
                ),
                owning_span,
                comment,
            },
        ];
    }

    // This is a jump backwards. The approaches are same as forward jumping, but
    // the version with only a single instruction is not possible, and we have to replace
    // ADD with SUB instructions. A SUBI-based approach can be used, for a cheap 2-instruction case.
    let delta = curr_offset - target_offset;

    // Attempt SUBI-based approach
    if let Ok(imm) = VirtualImmediate12::try_new(
        delta.saturating_mul(Instruction::SIZE as u64),
        Span::dummy(),
    ) {
        return vec![
            RealizedOp {
                opcode: AllocatedInstruction::SUBI(
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                    AllocatedRegister::Constant(ConstantRegister::ProgramCounter),
                    imm,
                ),
                owning_span: owning_span.clone(),
                comment: "load far jump target address".into(),
            },
            RealizedOp {
                opcode: AllocatedInstruction::JAL(
                    AllocatedRegister::Constant(ConstantRegister::CallReturnAddress),
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                    VirtualImmediate12::new(0),
                ),
                owning_span,
                comment,
            },
        ];
    }

    // Since the rest of the approaches require an extra instruction before the PC is used, we
    // add 1 to the delta to account for that. It cannot underflow as otherwise the first
    // approach would have been used. Then we multiply by instruction size for doing arithmetic
    // with the PC register. The overflow cannot occur since programs cannot be 2**60 bytes large.
    let delta_instr = (delta + 1) * (Instruction::SIZE as u64);

    // Attempt MOVI-based approach.
    if let Ok(imm) = VirtualImmediate18::try_new(delta_instr, Span::dummy()) {
        return vec![
            RealizedOp {
                opcode: AllocatedInstruction::MOVI(
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                    imm,
                ),
                owning_span: owning_span.clone(),
                comment: "load call target address".into(),
            },
            RealizedOp {
                opcode: AllocatedInstruction::SUB(
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                    AllocatedRegister::Constant(ConstantRegister::ProgramCounter),
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                ),
                owning_span: owning_span.clone(),
                comment: "load call target address".into(),
            },
            RealizedOp {
                opcode: AllocatedInstruction::JAL(
                    AllocatedRegister::Constant(ConstantRegister::CallReturnAddress),
                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                    VirtualImmediate12::new(0),
                ),
                owning_span,
                comment,
            },
        ];
    }

    // And lastly, fall back to the data section backed approach.
    let data_id = data_section.insert_data_value(Entry::new_word(
        delta_instr,
        EntryName::NonConfigurable,
        None,
    ));

    vec![
        RealizedOp {
            opcode: AllocatedInstruction::LoadDataId(
                AllocatedRegister::Constant(ConstantRegister::Scratch),
                data_id,
            ),
            owning_span: owning_span.clone(),
            comment: "load call target address".into(),
        },
        RealizedOp {
            opcode: AllocatedInstruction::SUB(
                AllocatedRegister::Constant(ConstantRegister::Scratch),
                AllocatedRegister::Constant(ConstantRegister::ProgramCounter),
                AllocatedRegister::Constant(ConstantRegister::Scratch),
            ),
            owning_span: owning_span.clone(),
            comment: "load call target address".into(),
        },
        RealizedOp {
            opcode: AllocatedInstruction::JAL(
                AllocatedRegister::Constant(ConstantRegister::CallReturnAddress),
                AllocatedRegister::Constant(ConstantRegister::Scratch),
                VirtualImmediate12::new(0),
            ),
            owning_span,
            comment,
        },
    ]
}

/// Compiles a function call into the appropriate operations.
/// Pads the call to the size reserved for it.
pub(crate) fn compile_call(
    data_section: &mut DataSection,
    curr_offset: u64,
    target_offset: u64,
    far_size: Option<u64>,
    comment: String,
    owning_span: Option<Span>,
) -> Vec<RealizedOp> {
    let mut res = compile_call_inner(
        data_section,
        curr_offset,
        target_offset,
        comment.clone(),
        owning_span.clone(),
    );
    while res.len() < far_size.unwrap_or(1) as usize {
        res.push(RealizedOp {
            opcode: AllocatedInstruction::NOOP,
            owning_span: owning_span.clone(),
            comment: comment.clone(),
        });
    }
    res
}
