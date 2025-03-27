use crate::{
    asm_generation::fuel::data_section::EntryName,
    asm_lang::{
        allocated_ops::{AllocatedOpcode, AllocatedRegister}, AllocatedAbstractOp, ConstantRegister, ControlFlowOp, Label, LoadLabelType, RealizedOp, VirtualImmediate12, VirtualImmediate18, VirtualImmediate24
    },
};

use super::{
    abstract_instruction_set::RealizedAbstractInstructionSet,
    compiler_constants as consts,
    data_section::{DataSection, Entry},
};

use fuel_vm::fuel_asm::Imm12;
use indexmap::{IndexMap, IndexSet};
use sway_types::span::Span;

use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap, HashSet},
};

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
    pub(crate) fn optimize(self) -> AllocatedAbstractInstructionSet {
        self.remove_redundant_ops()
    }

    fn remove_redundant_ops(mut self) -> AllocatedAbstractInstructionSet {
        self.ops.retain(|op| {
            // It is easier to think in terms of operations we want to remove
            // than the operations we want to retain ;-)
            let remove = match &op.opcode {
                // `cfei i0` and `cfsi i0` pairs.
                Either::Left(AllocatedOpcode::CFEI(imm))
                | Either::Left(AllocatedOpcode::CFSI(imm)) => imm.value() == 0u32,
                // `cfe $zero` and `cfs $zero` pairs.
                Either::Left(AllocatedOpcode::CFE(reg))
                | Either::Left(AllocatedOpcode::CFS(reg)) => reg.is_zero(),
                _ => false,
            };

            !remove
        });

        self
    }

    /// Replace each PUSHA instruction with stores of all used registers to the stack, and each
    /// POPA with respective loads from the stack.
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
                VirtualImmediate24::new(mask.0, Span::dummy())
                    .expect("mask should have fit in 24b"),
                VirtualImmediate24::new(mask.1, Span::dummy())
                    .expect("mask should have fit in 24b"),
            )
        }

        // Now replace the PUSHA/POPA instructions with STOREs and LOADs.
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
                            opcode: Either::Left(AllocatedOpcode::PSHL(mask_l)),
                            comment: "save registers 16..40".into(),
                            owning_span: op.owning_span.clone(),
                        });
                    }
                    if mask_h.value() != 0 {
                        new_ops.push(AllocatedAbstractOp {
                            opcode: Either::Left(AllocatedOpcode::PSHH(mask_h)),
                            comment: "save registers 40..64".into(),
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
                            opcode: Either::Left(AllocatedOpcode::POPH(mask_h)),
                            comment: "restore registers 40..64".into(),
                            owning_span: op.owning_span.clone(),
                        });
                    }
                    if mask_l.value() != 0 {
                        new_ops.push(AllocatedAbstractOp {
                            opcode: Either::Left(AllocatedOpcode::POPL(mask_l)),
                            comment: "restore registers 16..40".into(),
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
    ) -> Result<(RealizedAbstractInstructionSet, LabeledBlocks), crate::CompileError> {
        let label_offsets = self.resolve_labels(data_section, 0)?;
        let mut curr_offset = 0;

        let mut realized_ops = vec![];
        for (op_idx, op) in self.ops.iter().enumerate() {
            let op_size = Self::instruction_size(op, data_section);
            let rel_offset =
                |curr_offset, lab| label_offsets.get(lab).unwrap().offs.abs_diff(curr_offset);
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
                    ControlFlowOp::Jump(ref lab) => {
                        let imm = || {
                            VirtualImmediate18::new_unchecked(
                                // JMP(B/F) adds a 1
                                rel_offset(curr_offset, lab) - 1,
                                "Programs with more than 2^18 labels are unsupported right now",
                            )
                        };
                        match curr_offset.cmp(&label_offsets.get(lab).unwrap().offs) {
                            Ordering::Equal => {
                                assert!(matches!(
                                    self.ops[op_idx - 1].opcode,
                                    Either::Left(AllocatedOpcode::NOOP)
                                ));
                                realized_ops.push(RealizedOp {
                                    opcode: AllocatedOpcode::JMPB(
                                        AllocatedRegister::Constant(ConstantRegister::Zero),
                                        VirtualImmediate18::new_unchecked(0, "unreachable()"),
                                    ),
                                    owning_span,
                                    comment,
                                });
                            }
                            Ordering::Greater => {
                                realized_ops.push(RealizedOp {
                                    opcode: AllocatedOpcode::JMPB(
                                        AllocatedRegister::Constant(ConstantRegister::Zero),
                                        imm(),
                                    ),
                                    owning_span,
                                    comment,
                                });
                            }
                            Ordering::Less => {
                                realized_ops.push(RealizedOp {
                                    opcode: AllocatedOpcode::JMPF(
                                        AllocatedRegister::Constant(ConstantRegister::Zero),
                                        imm(),
                                    ),
                                    owning_span,
                                    comment,
                                });
                            }
                        }
                    }
                    ControlFlowOp::JumpIfNotZero(r1, ref lab) => {
                        let imm = || {
                            VirtualImmediate12::new_unchecked(
                                // JNZ(B/F) adds a 1
                                rel_offset(curr_offset, lab) - 1,
                                "Programs with more than 2^12 labels are unsupported right now",
                            )
                        };
                        match curr_offset.cmp(&label_offsets.get(lab).unwrap().offs) {
                            Ordering::Equal => {
                                assert!(matches!(
                                    self.ops[op_idx - 1].opcode,
                                    Either::Left(AllocatedOpcode::NOOP)
                                ));
                                realized_ops.push(RealizedOp {
                                    opcode: AllocatedOpcode::JNZB(
                                        r1,
                                        AllocatedRegister::Constant(ConstantRegister::Zero),
                                        VirtualImmediate12::new_unchecked(0, "unreachable()"),
                                    ),
                                    owning_span,
                                    comment,
                                });
                            }
                            Ordering::Greater => {
                                realized_ops.push(RealizedOp {
                                    opcode: AllocatedOpcode::JNZB(
                                        r1,
                                        AllocatedRegister::Constant(ConstantRegister::Zero),
                                        imm(),
                                    ),
                                    owning_span,
                                    comment,
                                });
                            }
                            Ordering::Less => {
                                realized_ops.push(RealizedOp {
                                    opcode: AllocatedOpcode::JNZF(
                                        r1,
                                        AllocatedRegister::Constant(ConstantRegister::Zero),
                                        imm(),
                                    ),
                                    owning_span,
                                    comment,
                                });
                            }
                        }
                    }
                    ControlFlowOp::Call(ref lab) => {
                        // rewrite_far_jumps guarantees our label can be jumped to using a single jal
                        let offs = label_offsets.get(lab).unwrap().offs;
                        assert!(offs > curr_offset);
                        assert!(offs - curr_offset <= consts::TWELVE_BITS, "{curr_offset} - {offs}");
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::JAL(
                                AllocatedRegister::Constant(ConstantRegister::CallReturnAddress),
                                AllocatedRegister::Constant(ConstantRegister::ProgramCounter),
                                VirtualImmediate12::new_unchecked(offs - curr_offset, "rewrite_far_jumps makes this unreachable"),
                            ),
                            owning_span,
                            comment,
                        });
                    }
                    ControlFlowOp::DataSectionOffsetPlaceholder => {
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::DataSectionOffsetPlaceholder,
                            owning_span: None,
                            comment: String::new(),
                        });
                    }
                    ControlFlowOp::ConfigurablesOffsetPlaceholder => {
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::ConfigurablesOffsetPlaceholder,
                            owning_span: None,
                            comment: String::new(),
                        });
                    }
                    ControlFlowOp::LoadLabel(r1, ref lab, type_) => {
                        // LoadLabel ops are inserted by `rewrite_far_jumps`,
                        // So the next instruction must be a jump.
                        match type_ {
                            LoadLabelType::Relative => {
                                assert!(matches!(
                                    self.ops[op_idx + 1].opcode,
                                    Either::Left(
                                        AllocatedOpcode::JMPB(..)
                                            | AllocatedOpcode::JNZB(..)
                                            | AllocatedOpcode::JMPF(..)
                                            | AllocatedOpcode::JNZF(..)
                                    )
                                ));

                                // Sub 1 because the relative jumps add a 1.
                                let offset = rel_offset(curr_offset + 1, lab) - 1;
                                let data_id = data_section.insert_data_value(Entry::new_word(
                                    offset,
                                    EntryName::NonConfigurable,
                                    None,
                                ));
                                realized_ops.push(RealizedOp {
                                    opcode: AllocatedOpcode::LoadDataId(r1, data_id),
                                    owning_span,
                                    comment,
                                });
                            },
                            LoadLabelType::JAL => {
                                // Note that we assume a forward jump here.
                                // When inserting this, backwards jumps insert SUB instruction to get the negative value.
                                assert!(matches!(
                                    self.ops[op_idx + 1].opcode,
                                    Either::Left(
                                        AllocatedOpcode::JAL(..)
                                        // SUB used for backwards jumps
                                        | AllocatedOpcode::SUB(..)
                                    )
                                ));
                                let offset = rel_offset(curr_offset, lab);
                                let data_id = data_section.insert_data_value(Entry::new_word(
                                    offset,
                                    EntryName::NonConfigurable,
                                    None,
                                ));
                                realized_ops.push(RealizedOp {
                                    opcode: AllocatedOpcode::LoadDataId(r1, data_id),
                                    owning_span,
                                    comment,
                                });
                            }
                        };
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

    fn resolve_labels(
        &mut self,
        data_section: &mut DataSection,
        iter_count: usize,
    ) -> Result<LabeledBlocks, crate::CompileError> {
        // Iteratively resolve the label offsets.
        //
        // For very large programs the label offsets may be too large to fit in an immediate jump
        // (JI, JNEI or JNZI).  In these case we must replace the immediate jumps with register
        // based jumps (JMP, JNE) but these require more than one instruction; usually an
        // instruction to load the destination register and then the jump itself.
        //
        // But we don't know the offset of a label until we scan through the ops and count them.
        // So we have a chicken and egg situation where we may need to add new instructions which
        // would change the offsets to all labels thereafter, which in turn could require more
        // instructions to be added, and so on.
        //
        // This should really only take 2 iterations (and only for very, very large programs) and
        // the pathological case somehow has many labels clustered at the 12 or 18 bit boundaries
        // which switch from immediate to register based destinations after each loop.

        if iter_count > 10 {
            return Err(crate::CompileError::Internal(
                "Failed to resolve ASM label offsets.",
                Span::dummy(),
            ));
        }

        let (remap_needed, label_offsets) = self.map_label_offsets(data_section);

        if !remap_needed || !self.rewrite_far_jumps(&label_offsets, data_section) {
            // We didn't need to make any changes to the ops, so the labels are now correct.
            Ok(label_offsets)
        } else {
            // We did add new ops and so we need to update the label offsets.
            self.resolve_labels(data_section, iter_count + 1)
        }
    }

    // Instruction size in units of 32b.
    fn instruction_size(op: &AllocatedAbstractOp, data_section: &DataSection) -> u64 {
        use ControlFlowOp::*;
        match op.opcode {
            Either::Right(Label(_)) => 0,

            // A special case for LoadDataId which may be 1 or 2 ops, depending on the source size.
            Either::Left(AllocatedOpcode::LoadDataId(_, ref data_id)) => {
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

            Either::Left(AllocatedOpcode::AddrDataId(_, ref id)) => {
                if data_section.data_id_to_offset(id) > usize::from(Imm12::MAX.to_u16()) {
                    2
                } else {
                    1
                }
            }

            // cfei 0 and cfsi 0 are omitted from asm emission, don't count them for offsets
            Either::Left(AllocatedOpcode::CFEI(ref op))
            | Either::Left(AllocatedOpcode::CFSI(ref op))
                if op.value() == 0 =>
            {
                0
            }

            // Another special case for the blob opcode, used for testing.
            Either::Left(AllocatedOpcode::BLOB(ref count)) => count.value() as u64,

            // These ops will end up being exactly one op, so the cur_offset goes up one.
            Either::Right(Jump(..) | JumpIfNotZero(..) | Call { .. } | LoadLabel(..))
            | Either::Left(_) => 1,

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

    fn map_label_offsets(&self, data_section: &DataSection) -> (bool, LabeledBlocks) {
        let mut labelled_blocks = LabeledBlocks::new();
        let mut cur_offset = 0;
        let mut cur_basic_block = None;

        // We decide here whether remapping jumps are necessary.
        // 1. JMPB and JMPF offsets are more than 18 bits
        // 2. JNZF and JNZB offsets are more than 12 bits
        // 3. JAL offset is more than 12 bits or negative

        let mut jnz_labels = HashSet::new();
        let mut jmp_labels = HashSet::new();
        let mut jal_labels = HashSet::new();

        use ControlFlowOp::*;

        for (op_idx, op) in self.ops.iter().enumerate() {
            // If we're seeing a control flow op then it's the end of the block.
            if let Either::Right(Label(_) | Jump(_) | JumpIfNotZero(..)) = op.opcode {
                if let Some((lab, _idx, offs)) = cur_basic_block {
                    // Insert the previous basic block.
                    labelled_blocks.insert(lab, BasicBlock { offs });
                }
            }

            if let Either::Right(Label(cur_lab)) = op.opcode {
                // Save the new block label and furthest offset.
                cur_basic_block = Some((cur_lab, op_idx, cur_offset));
            }

            if let Either::Right(Jump(lab)) = op.opcode {
                jmp_labels.insert((cur_offset, lab));
            }

            if let Either::Right(JumpIfNotZero(_, lab)) = op.opcode {
                jnz_labels.insert((cur_offset, lab));
            }

            if let Either::Right(Call(lab)) = op.opcode {
                jal_labels.insert((cur_offset, lab));
            }

            // Update the offset.
            cur_offset += Self::instruction_size(op, data_section);
        }

        // Don't forget the final block.
        if let Some((lab, _idx, offs)) = cur_basic_block {
            labelled_blocks.insert(lab, BasicBlock { offs });
        }

        let rel_needs_remap = |offset, lab, limit| {
            let rel_offset = labelled_blocks.get(lab).unwrap().offs.abs_diff(offset);
            // Self jumps need a NOOP inserted before it so that we can jump to the NOOP.
            // if rel_offset exceeds limit, we'll need to insert LoadLabels.
            rel_offset == 0 || rel_offset > limit
        };
        let jal_needs_remap = |offset, lab| {
            let offs = labelled_blocks.get(lab).unwrap().offs;
            offs < offset || (offs - offset) > consts::TWELVE_BITS
        };
        let need_to_remap_jumps = jmp_labels
            .iter()
            .any(|(offset, lab)| rel_needs_remap(*offset, lab, consts::EIGHTEEN_BITS))
            || jnz_labels
                .iter()
                .any(|(offset, lab)| rel_needs_remap(*offset, lab, consts::TWELVE_BITS))
            || jal_labels
                .iter()
                .any(|(offset, lab)| jal_needs_remap(*offset, lab));

        (need_to_remap_jumps, labelled_blocks)
    }

    /// If an instruction uses a label which can't fit in its immediate value then translate it
    /// into an instruction which loads the offset from the data section into a register and then
    /// use the equivalent non-immediate instruction with the register.
    fn rewrite_far_jumps(
        &mut self,
        label_offsets: &LabeledBlocks,
        data_section: &DataSection,
    ) -> bool {
        let min_ops = self.ops.len();
        let mut modified = false;
        let mut curr_offset = 0;

        self.ops = self
            .ops
            .drain(..)
            .fold(Vec::with_capacity(min_ops), |mut new_ops, op| {
                let op_size = Self::instruction_size(&op, data_section);
                let rel_offset = |lab| label_offsets.get(lab).unwrap().offs.abs_diff(curr_offset);
                match &op.opcode {
                    Either::Right(ControlFlowOp::Jump(ref lab)) => {
                        if rel_offset(lab) == 0 {
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Left(AllocatedOpcode::NOOP),
                                comment: "emit noop for self loop".into(),
                                owning_span: None,
                            });
                            new_ops.push(op);
                            modified = true;
                        } else if rel_offset(lab) - 1 <= consts::EIGHTEEN_BITS {
                            new_ops.push(op)
                        } else {
                            // Load the offset into $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Right(ControlFlowOp::LoadLabel(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    *lab,
                                    LoadLabelType::Relative,
                                )),
                                comment: String::new(),
                                owning_span: None,
                            });

                            // Jump to $tmp.
                            if curr_offset > label_offsets.get(lab).unwrap().offs {
                                new_ops.push(AllocatedAbstractOp {
                                    opcode: Either::Left(AllocatedOpcode::JMPB(
                                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                                        VirtualImmediate18::new_unchecked(0, "zero must fit in 18 bits"),
                                    )),
                                    ..op
                                });
                            } else {
                                new_ops.push(AllocatedAbstractOp {
                                    opcode: Either::Left(AllocatedOpcode::JMPF(
                                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                                        VirtualImmediate18 ::new_unchecked(0, "zero must fit in 18 bits"),
                                    )),
                                    ..op
                                });
                            }
                            modified = true;
                        }
                    }
                    Either::Right(ControlFlowOp::JumpIfNotZero(r1, ref lab)) => {
                        if rel_offset(lab) == 0 {
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Left(AllocatedOpcode::NOOP),
                                comment: "emit noop for self loop".into(),
                                owning_span: None,
                            });
                            new_ops.push(op);
                            modified = true;
                        } else if rel_offset(lab) - 1 <= consts::TWELVE_BITS {
                            new_ops.push(op)
                        } else {
                            // Load the destination address into $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Right(ControlFlowOp::LoadLabel(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    *lab,
                                    LoadLabelType::Relative,
                                )),
                                comment: String::new(),
                                owning_span: None,
                            });

                            // JNZB/JNZF r1 $tmp.
                            if curr_offset > label_offsets.get(lab).unwrap().offs {
                                new_ops.push(AllocatedAbstractOp {
                                    opcode: Either::Left(AllocatedOpcode::JNZB(
                                        r1.clone(),
                                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                                        VirtualImmediate12::new_unchecked(0, "zero must fit in 12 bits"),
                                    )),
                                    ..op
                                });
                            } else {
                                new_ops.push(AllocatedAbstractOp {
                                    opcode: Either::Left(AllocatedOpcode::JNZF(
                                        r1.clone(),
                                        AllocatedRegister::Constant(ConstantRegister::Scratch),
                                        VirtualImmediate12::new_unchecked(0, "zero must fit in 12 bits"),
                                    )),
                                    ..op
                                });
                            }
                            modified = true;
                        }
                    }
                    Either::Right(ControlFlowOp::Call(ref lab)) => {
                        let lab_offs = label_offsets.get(lab).unwrap().offs;
                        if lab_offs < curr_offset {
                            // We use `JAL` instruciton to do calls. It doesn't support backwards offsets.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Right(ControlFlowOp::LoadLabel(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    *lab,
                                    LoadLabelType::JAL,
                                )),
                                comment: String::new(),
                                owning_span: None,
                            });
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Left(AllocatedOpcode::SUB(
                                    AllocatedRegister::Constant(ConstantRegister::ProgramCounter),
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    AllocatedRegister::Constant(ConstantRegister::ProgramCounter),
                                )),
                                comment: String::new(),
                                owning_span: None,
                            });
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Left(AllocatedOpcode::JAL (
                                    AllocatedRegister::Constant(ConstantRegister::CallReturnAddress),
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    VirtualImmediate12::new_unchecked(0, "zero must fit in 12 bits"),
                                )),
                                ..op
                            });
                            modified = true;
                        } else if lab_offs > consts::EIGHTEEN_BITS {
                            // Offset cannot fit into MOVI, do full load.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Right(ControlFlowOp::LoadLabel(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    *lab,
                                    LoadLabelType::JAL,
                                )),
                                comment: String::new(),
                                owning_span: None,
                            });
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Left(AllocatedOpcode::JAL (
                                    AllocatedRegister::Constant(ConstantRegister::CallReturnAddress),
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    VirtualImmediate12::new_unchecked(0, "zero must fit in 12 bits"),
                                )),
                                ..op
                            });
                            modified = true;
                        } else if lab_offs > consts::TWELVE_BITS {
                            // We can use MOVI for this
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Left(AllocatedOpcode::MOVI(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    VirtualImmediate18::new_unchecked(lab_offs, "label offset must fit in 12 bits"),
                                )),
                                comment: String::new(),
                                owning_span: None,
                            });
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Left(AllocatedOpcode::JAL (
                                    AllocatedRegister::Constant(ConstantRegister::CallReturnAddress),
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    VirtualImmediate12::new_unchecked(0, "zero must fit in 12 bits"),
                                )),
                                ..op
                            });
                            modified = true;
                        } else {
                            // This fits as-is
                            new_ops.push(op);
                        }
                    }

                    // Everything else we copy as is.
                    _ => new_ops.push(op),
                }
                curr_offset += op_size;
                new_ops
            });

        modified
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
