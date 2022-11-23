use crate::asm_lang::{
    allocated_ops::{AllocatedOpcode, AllocatedRegister},
    AllocatedAbstractOp, ConstantRegister, ControlFlowOp, Label, RealizedOp, VirtualImmediate12,
    VirtualImmediate18, VirtualImmediate24,
};

use super::{compiler_constants as consts, DataSection, Entry, RealizedAbstractInstructionSet};

use sway_types::span::Span;

use std::collections::{BTreeSet, HashMap, HashSet};

use either::Either;

// Convenience type for representing a map from a label to its offset and number of instructions
// following it until the next label (i.e., the length of the basic block).
pub(crate) type LabeledBlocks = HashMap<Label, BasicBlock>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct BasicBlock {
    pub(crate) offs: u64,
    pub(crate) abstract_len: usize,
    pub(crate) final_len: u64,
}

#[derive(Clone)]
pub struct AllocatedAbstractInstructionSet {
    pub(crate) ops: Vec<AllocatedAbstractOp>,
}

impl AllocatedAbstractInstructionSet {
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
                (HashMap::new(), HashSet::new()),
                |(mut reg_sets, mut active_sets), op| {
                    let reg = match &op.opcode {
                        Either::Right(ControlFlowOp::PushAll(label)) => {
                            active_sets.insert(*label);
                            None
                        }
                        Either::Right(ControlFlowOp::PopAll(label)) => {
                            active_sets.remove(label);
                            None
                        }

                        Either::Left(alloc_op) => alloc_op.def_registers().into_iter().next(),
                        Either::Right(ctrl_op) => ctrl_op.def_registers().into_iter().next(),
                    };

                    if let Some(reg) = reg {
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

        // Now replace the PUSHA/POPA instructions with STOREs and LOADs.
        self.ops = self.ops.drain(..).fold(Vec::new(), |mut new_ops, op| {
            match &op.opcode {
                Either::Right(ControlFlowOp::PushAll(label)) => {
                    let regs = reg_sets
                        .get(label)
                        .expect("Have collected registers above.")
                        .iter()
                        .filter(|reg| matches!(reg, AllocatedRegister::Allocated(_)))
                        .collect::<Vec<_>>();

                    let stack_use_bytes = regs.len() as u64 * 8;
                    new_ops.push(AllocatedAbstractOp {
                        opcode: Either::Left(AllocatedOpcode::MOVE(
                            AllocatedRegister::Constant(ConstantRegister::Scratch),
                            AllocatedRegister::Constant(ConstantRegister::StackPointer),
                        )),
                        comment: "save base stack value".into(),
                        owning_span: None,
                    });
                    new_ops.push(AllocatedAbstractOp {
                        opcode: Either::Left(AllocatedOpcode::CFEI(
                            VirtualImmediate24::new(stack_use_bytes, Span::dummy()).unwrap(),
                        )),
                        comment: "reserve space for saved registers".into(),
                        owning_span: None,
                    });

                    regs.into_iter().enumerate().for_each(|(idx, reg)| {
                        let store_op = AllocatedOpcode::SW(
                            AllocatedRegister::Constant(ConstantRegister::Scratch),
                            reg.clone(),
                            VirtualImmediate12::new(idx as u64, Span::dummy()).unwrap(),
                        );
                        new_ops.push(AllocatedAbstractOp {
                            opcode: Either::Left(store_op),
                            comment: format!("save {}", reg),
                            owning_span: None,
                        });
                    })
                }

                Either::Right(ControlFlowOp::PopAll(label)) => {
                    let regs = reg_sets
                        .get(label)
                        .expect("Have collected registers above.")
                        .iter()
                        .filter(|reg| matches!(reg, AllocatedRegister::Allocated(_)))
                        .collect::<Vec<_>>();

                    let stack_use_bytes = regs.len() as u64 * 8;
                    new_ops.push(AllocatedAbstractOp {
                        opcode: Either::Left(AllocatedOpcode::SUBI(
                            AllocatedRegister::Constant(ConstantRegister::Scratch),
                            AllocatedRegister::Constant(ConstantRegister::StackPointer),
                            VirtualImmediate12::new(stack_use_bytes, Span::dummy()).unwrap(),
                        )),
                        comment: "save base stack value".into(),
                        owning_span: None,
                    });

                    regs.into_iter().enumerate().for_each(|(idx, reg)| {
                        let load_op = AllocatedOpcode::LW(
                            reg.clone(),
                            AllocatedRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate12::new(idx as u64, Span::dummy()).unwrap(),
                        );
                        new_ops.push(AllocatedAbstractOp {
                            opcode: Either::Left(load_op),
                            comment: format!("restore {}", reg),
                            owning_span: None,
                        });
                    });

                    new_ops.push(AllocatedAbstractOp {
                        opcode: Either::Left(AllocatedOpcode::CFSI(
                            VirtualImmediate24::new(stack_use_bytes, Span::dummy()).unwrap(),
                        )),
                        comment: "recover space from saved registers".into(),
                        owning_span: None,
                    });
                }

                _otherwise => new_ops.push(op),
            };
            new_ops
        });

        self
    }

    /// Relocate code to keep as much control flow possible using a `JNZI` instruction.
    ///
    /// If a `JNZI` or `JNEI` instruction would be unable to store its destination offset in its
    /// immediate operand then we have two remediation options; relocate blocks which don't use
    /// control flow to the end of the address space, or to store the large offsets in the data
    /// section and use dynamic jumps.  This function performs the former transformation while
    /// `rewrite_far_jumps()` performs the latter.
    ///
    /// For example, if a bytecode binary reaches more than 1MB in size then to perform conditional
    /// controlflow using `JNZI` the destination offset may be to an instruction index of more than
    /// 256K, as each instruction is a 32bit word.  Values larger than 256K require 19bits of
    /// representation which is too large to fit in the 18bit immediate operand of `JNZI`.
    ///
    /// Here we use the convenience of `JI` being able to jump to a 24bit offset.  Say we had the
    /// following bytecode straddling the 256K instruction boundary:
    ///
    /// ```text
    /// 0003fffd JNZI label_A       ; jump to symbolic offset at label_A
    /// 0003fffe NOOP               ; very important work
    /// 0003ffff NOOP
    /// 00040000 NOOP
    /// 00040001 NOOP
    ///          label_A:           ; label which turns out to be at offset 256K + 2
    /// 00040002 RET 42             ; return the 'answer'.
    /// ```
    ///
    /// We would not be able to encode the `JNZI` because `label_A` is too far.
    /// `relocate_control_flow()` would transform this bytecode into the following to fix the
    /// problem:
    ///
    /// ```text
    /// 0003fffd JNZI label_A       ; jump to symbolic offset at label_A
    /// 0003fffe JI label_B         ; jump to the important work with a 24bit offset.
    ///          label_A:           ; label which is now (just) below 256K.
    /// 0003ffff RET 42             ; return the 'answer'.
    ///          label_B:
    /// 00040000 NOOP               ; very important work
    /// 00040001 NOOP
    /// 00040002 NOOP
    /// 00040003 NOOP
    /// 00040004 JI label_A         ; jump back now we're done.
    /// ```
    pub(crate) fn relocate_control_flow(mut self, data_section: &DataSection) -> Self {
        // Do an analysis pass, gathering basic block offsets and whether any jumps are going to be
        // a problem.
        let (has_far_jumps, furthest_offset, blocks) = self.map_label_offsets(data_section);

        if !has_far_jumps {
            return self;
        }

        // Sort the blocks by _final_ size, biggest first.
        let mut sorted_blocks = blocks.into_iter().collect::<Vec<_>>();
        sorted_blocks.sort_unstable_by(|l, r| l.1.final_len.cmp(&r.1.final_len).reverse());

        let reduction_target = furthest_offset - consts::EIGHTEEN_BITS;

        // We create a map of blocks to move but storing the _abstract_ size of the block.
        let mut instructions_to_move_count = 0;
        let blocks_to_move: HashMap<Label, usize> = HashMap::from_iter(
            sorted_blocks
                .into_iter()
                .take_while(|(_lab, blk)| {
                    let keep_going = instructions_to_move_count < reduction_target;
                    instructions_to_move_count += blk.final_len;
                    keep_going
                })
                .map(|(lab, blk)| (lab, blk.abstract_len)),
        );

        // This is all very imperative, but we're trying to keep it efficient.  We expect the
        // number of instructions in the `moved_ops` list to be `instructions_to_move_count` plus a
        // label op for each block which is moved.
        let mut new_ops: Vec<AllocatedAbstractOp> = Vec::with_capacity(self.ops.len());
        let mut moved_ops: Vec<AllocatedAbstractOp> =
            Vec::with_capacity(blocks_to_move.len() + instructions_to_move_count as usize);

        // A util function to wrap the new control flow opcodes.
        let mk_op = |opcode| AllocatedAbstractOp {
            opcode: Either::Right(opcode),
            comment: String::new(),
            owning_span: None,
        };

        // Use a large number for our new moved labels, one which shouldn't be in the existing ops.
        // This is a little bit hacky. :/
        let mut new_label_idx = self.ops.len();

        let mut read_idx = 0;
        while read_idx < self.ops.len() {
            // Get the next op and copy it over.
            let op = &self.ops[read_idx];
            new_ops.push(op.clone());
            read_idx += 1;

            // Check if this is a label and one for a block we want to move...
            if let Either::Right(ControlFlowOp::Label(ref cur_label)) = op.opcode {
                if let Some(count) = blocks_to_move.get(cur_label) {
                    // The count in the set includes the label.
                    let count = *count as usize - 1;

                    // We want to move this block.  First add a new label to the moved list for us
                    // to jump to.
                    let moved_block_label = Label(new_label_idx);
                    new_label_idx += 1;
                    moved_ops.push(mk_op(ControlFlowOp::Label(moved_block_label)));

                    // Now add all the instruction in this block to the moved list.
                    moved_ops.extend_from_slice(&self.ops[read_idx..(read_idx + count)]);

                    // Replace the block with a jump to the new label.
                    new_ops.push(mk_op(ControlFlowOp::Jump(moved_block_label)));

                    // Move the `read_idx` to beyond the block we just moved.
                    read_idx += count;

                    // Now check this next op.  If it's a label then we need the moved block to
                    // jump back to it.  If it's an unconditional jump then it can also be moved to
                    // the end of the moved block.  Conditional jumps must not be moved so we add a
                    // new label to jump back to.  I.e., we need to terminate the moved block.
                    match &self.ops[read_idx].opcode {
                        Either::Right(ControlFlowOp::Label(return_lab)) => {
                            // Do _not_ increment read_idx; we need to copy this label to the
                            // `new_ops` next iteration.
                            moved_ops.push(mk_op(ControlFlowOp::Jump(*return_lab)));
                        }

                        Either::Right(ControlFlowOp::JumpIfNotEq(..))
                        | Either::Right(ControlFlowOp::JumpIfNotZero(..)) => {
                            // Also don't increment read_idx to let it be copied next iteration.
                            let jump_back_label = Label(new_label_idx);
                            new_label_idx += 1;
                            new_ops.push(mk_op(ControlFlowOp::Label(moved_block_label)));
                            moved_ops.push(mk_op(ControlFlowOp::Label(jump_back_label)));
                        }

                        Either::Right(ControlFlowOp::Jump(_)) => {
                            // Here we _do_ increment read_idx as we want this instruction to be
                            // counted as a part of the moved block.
                            moved_ops.push(self.ops[read_idx].clone());
                            read_idx += 1;
                        }

                        _otherwise => unreachable!(
                            "We should not have basic blocks terminated by any other instruction.\
                            See map_label_offsets() below."
                        ),
                    }
                }
            }
        }

        new_ops.append(&mut moved_ops);
        self.ops = new_ops;

        self
    }

    /// Runs two passes -- one to get the instruction offsets of the labels and one to replace the
    /// labels in the organizational ops
    pub(crate) fn realize_labels(
        mut self,
        data_section: &mut DataSection,
    ) -> Result<(RealizedAbstractInstructionSet, LabeledBlocks), crate::CompileError> {
        let label_offsets = self.resolve_labels(data_section, 0)?;

        let mut realized_ops = vec![];
        for AllocatedAbstractOp {
            opcode,
            comment,
            owning_span,
        } in self.ops.clone().into_iter()
        {
            match opcode {
                Either::Left(op) => realized_ops.push(RealizedOp {
                    opcode: op,
                    owning_span,
                    comment,
                }),
                Either::Right(org_op) => match org_op {
                    ControlFlowOp::Jump(ref lab) | ControlFlowOp::Call(ref lab) => {
                        let imm = VirtualImmediate24::new_unchecked(
                            label_offsets.get(lab).unwrap().offs,
                            "Programs with more than 2^24 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::JI(imm),
                            owning_span,
                            comment,
                        });
                    }
                    ControlFlowOp::JumpIfNotEq(r1, r2, ref lab) => {
                        let imm = VirtualImmediate12::new_unchecked(
                            label_offsets.get(lab).unwrap().offs,
                            "Programs with more than 2^12 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::JNEI(r1, r2, imm),
                            owning_span,
                            comment,
                        });
                    }
                    ControlFlowOp::JumpIfNotZero(r1, ref lab) => {
                        let imm = VirtualImmediate18::new_unchecked(
                            label_offsets.get(lab).unwrap().offs,
                            "Programs with more than 2^18 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::JNZI(r1, imm),
                            owning_span,
                            comment,
                        });
                    }
                    ControlFlowOp::MoveAddress(r1, ref lab) => {
                        let imm = VirtualImmediate18::new_unchecked(
                            label_offsets.get(lab).unwrap().offs,
                            "Programs with more than 2^18 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::MOVI(r1, imm),
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
                    ControlFlowOp::LoadLabel(r1, ref lab) => {
                        let offset = label_offsets.get(lab).unwrap().offs;
                        let data_id = data_section.insert_data_value(Entry::new_word(offset, None));
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::LWDataId(r1, data_id),
                            owning_span,
                            comment,
                        });
                    }
                    ControlFlowOp::Comment => continue,
                    ControlFlowOp::Label(..) => continue,

                    ControlFlowOp::PushAll(_) | ControlFlowOp::PopAll(_) => {
                        unreachable!("still don't belong in organisational ops")
                    }
                },
            };
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

        let (remap_needed, _, label_offsets) = self.map_label_offsets(data_section);

        if !remap_needed || !self.rewrite_far_jumps(&label_offsets) {
            // We didn't need to make any changes to the ops, so the labels are now correct.
            Ok(label_offsets)
        } else {
            // We did add new ops and so we need to update the label offsets.
            self.resolve_labels(data_section, iter_count + 1)
        }
    }

    fn map_label_offsets(&self, data_section: &DataSection) -> (bool, u64, LabeledBlocks) {
        let mut labelled_blocks = LabeledBlocks::new();
        let mut cur_offset = 0;
        let mut cur_basic_block = None;

        // We decide here whether remapping the jumps _may_ be necessary.  We assume that if any
        // label is further than 18bits then we'll probably have to remap JNZIs, and we track JNEIs
        // specifically since they can only jump 12bits but are pretty rare.
        let mut furthest_offset = 0;
        let mut jnei_labels = HashSet::new();

        use ControlFlowOp::*;

        for (op_idx, op) in self.ops.iter().enumerate() {
            // If we're seeing a control flow op then it's the end of the block.
            if let Either::Right(Label(_) | Jump(_) | JumpIfNotEq(..) | JumpIfNotZero(..)) =
                op.opcode
            {
                if let Some((lab, idx, offs)) = cur_basic_block {
                    // Insert the previous basic block.
                    labelled_blocks.insert(
                        lab,
                        BasicBlock {
                            offs,
                            abstract_len: op_idx - idx,
                            final_len: cur_offset - offs,
                        },
                    );
                }
            }

            // Update the offset.
            match op.opcode {
                Either::Right(Label(cur_lab)) => {
                    // Save the new block label and offset.
                    cur_basic_block = Some((cur_lab, op_idx, cur_offset));
                    furthest_offset = std::cmp::max(furthest_offset, cur_offset);
                }

                // A special case for LWDataId which may be 1 or 2 ops, depending on the source size.
                Either::Left(AllocatedOpcode::LWDataId(_, ref data_id)) => {
                    let has_copy_type = data_section.has_copy_type(data_id).expect(
                        "Internal miscalculation in data section -- \
                        data id did not match up to any actual data",
                    );
                    cur_offset += if has_copy_type { 1 } else { 2 };
                }

                // Another special case for the blob opcode, used for testing.
                Either::Left(AllocatedOpcode::BLOB(ref count)) => cur_offset += count.value as u64,

                // These ops will end up being exactly one op, so the cur_offset goes up one.
                Either::Right(
                    Jump(..) | JumpIfNotZero(..) | Call(..) | MoveAddress(..) | LoadLabel(..),
                )
                | Either::Left(_) => {
                    cur_offset += 1;
                }

                Either::Right(JumpIfNotEq(_, _, lab)) => {
                    jnei_labels.insert(lab);
                    cur_offset += 1;
                }

                Either::Right(Comment) => (),

                Either::Right(DataSectionOffsetPlaceholder) => {
                    // If the placeholder is 32 bits, this is 1. if 64, this should be 2. We use LW
                    // to load the data, which loads a whole word, so for now this is 2.
                    cur_offset += 2
                }

                Either::Right(PushAll(_)) | Either::Right(PopAll(_)) => unreachable!(
                    "fix me, pushall and popall don't really belong in control flow ops \
                        since they're not about control flow"
                ),
            }
        }

        // Don't forget the final block.
        if let Some((lab, idx, offs)) = cur_basic_block {
            labelled_blocks.insert(
                lab,
                BasicBlock {
                    offs,
                    abstract_len: self.ops.len() - idx,
                    final_len: cur_offset - offs,
                },
            );
        }

        let need_to_remap_jumps = furthest_offset > consts::EIGHTEEN_BITS
            || jnei_labels
                .iter()
                .any(|lab| labelled_blocks.get(lab).copied().unwrap().offs > consts::TWELVE_BITS);

        (need_to_remap_jumps, furthest_offset, labelled_blocks)
    }

    /// If an instruction uses a label which can't fit in its immediate value then translate it
    /// into an instruction which loads the offset from the data section into a register and then
    /// use the equivalent non-immediate instruction with the register.
    fn rewrite_far_jumps(&mut self, label_offsets: &LabeledBlocks) -> bool {
        let min_ops = self.ops.len();
        let mut modified = false;

        self.ops = self
            .ops
            .drain(..)
            .fold(Vec::with_capacity(min_ops), |mut new_ops, op| {
                match &op.opcode {
                    Either::Right(ControlFlowOp::Jump(ref lab))
                    | Either::Right(ControlFlowOp::Call(ref lab)) => {
                        let offset = label_offsets.get(lab).unwrap().offs;
                        if offset <= consts::TWENTY_FOUR_BITS {
                            new_ops.push(op)
                        } else {
                            // Load the destination address into $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Right(ControlFlowOp::LoadLabel(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    *lab,
                                )),
                                comment: String::new(),
                                owning_span: None,
                            });

                            // Jump to $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Left(AllocatedOpcode::JMP(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                )),
                                ..op
                            });
                        }
                    }
                    Either::Right(ControlFlowOp::JumpIfNotEq(r1, r2, ref lab)) => {
                        let offset = label_offsets.get(lab).unwrap().offs;
                        if offset <= consts::TWELVE_BITS {
                            new_ops.push(op)
                        } else {
                            // Load the destination address into $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Right(ControlFlowOp::LoadLabel(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    *lab,
                                )),
                                comment: String::new(),
                                owning_span: None,
                            });

                            // JNE r1 r2 $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Left(AllocatedOpcode::JNE(
                                    r1.clone(),
                                    r2.clone(),
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                )),
                                ..op
                            });
                            modified = true;
                        }
                    }
                    Either::Right(ControlFlowOp::JumpIfNotZero(r1, ref lab)) => {
                        let offset = label_offsets.get(lab).unwrap().offs;
                        if offset <= consts::EIGHTEEN_BITS {
                            new_ops.push(op)
                        } else {
                            // Load the destination address into $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Right(ControlFlowOp::LoadLabel(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    *lab,
                                )),
                                comment: String::new(),
                                owning_span: None,
                            });

                            // JNE r1 $zero $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Left(AllocatedOpcode::JNE(
                                    r1.clone(),
                                    AllocatedRegister::Constant(ConstantRegister::Zero),
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                )),
                                ..op
                            });
                            modified = true;
                        }
                    }
                    Either::Right(ControlFlowOp::MoveAddress(r1, ref lab)) => {
                        let offset = label_offsets.get(lab).unwrap().offs;
                        if offset <= consts::EIGHTEEN_BITS {
                            new_ops.push(op)
                        } else {
                            // Load the destination address into $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode: Either::Right(ControlFlowOp::LoadLabel(r1.clone(), *lab)),
                                ..op
                            });
                            modified = true;
                        }
                    }

                    // Everything else we copy as is.
                    _ => new_ops.push(op),
                }
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
