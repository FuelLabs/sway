use crate::asm_lang::{
    allocated_ops::{AllocatedOpcode, AllocatedRegister},
    AllocatedAbstractOp, ConstantRegister, ControlFlowOp, Label, RealizedOp, VirtualImmediate12,
    VirtualImmediate18, VirtualImmediate24,
};

use super::{compiler_constants as consts, DataSection, Entry, RealizedAbstractInstructionSet};

use sway_types::span::Span;

use std::collections::{BTreeSet, HashMap, HashSet};

use either::Either;

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
                        opcode:      Either::Left(AllocatedOpcode::MOVE(
                            AllocatedRegister::Constant(ConstantRegister::Scratch),
                            AllocatedRegister::Constant(ConstantRegister::StackPointer),
                        )),
                        comment:     "save base stack value".into(),
                        owning_span: None,
                    });
                    new_ops.push(AllocatedAbstractOp {
                        opcode:      Either::Left(AllocatedOpcode::CFEI(
                            VirtualImmediate24::new(stack_use_bytes, Span::dummy()).unwrap(),
                        )),
                        comment:     "reserve space for saved registers".into(),
                        owning_span: None,
                    });

                    regs.into_iter().enumerate().for_each(|(idx, reg)| {
                        let store_op = AllocatedOpcode::SW(
                            AllocatedRegister::Constant(ConstantRegister::Scratch),
                            reg.clone(),
                            VirtualImmediate12::new(idx as u64, Span::dummy()).unwrap(),
                        );
                        new_ops.push(AllocatedAbstractOp {
                            opcode:      Either::Left(store_op),
                            comment:     format!("save {}", reg),
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
                        opcode:      Either::Left(AllocatedOpcode::SUBI(
                            AllocatedRegister::Constant(ConstantRegister::Scratch),
                            AllocatedRegister::Constant(ConstantRegister::StackPointer),
                            VirtualImmediate12::new(stack_use_bytes, Span::dummy()).unwrap(),
                        )),
                        comment:     "save base stack value".into(),
                        owning_span: None,
                    });

                    regs.into_iter().enumerate().for_each(|(idx, reg)| {
                        let load_op = AllocatedOpcode::LW(
                            reg.clone(),
                            AllocatedRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate12::new(idx as u64, Span::dummy()).unwrap(),
                        );
                        new_ops.push(AllocatedAbstractOp {
                            opcode:      Either::Left(load_op),
                            comment:     format!("restore {}", reg),
                            owning_span: None,
                        });
                    });

                    new_ops.push(AllocatedAbstractOp {
                        opcode:      Either::Left(AllocatedOpcode::CFSI(
                            VirtualImmediate24::new(stack_use_bytes, Span::dummy()).unwrap(),
                        )),
                        comment:     "recover space from saved registers".into(),
                        owning_span: None,
                    });
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
    ) -> Result<RealizedAbstractInstructionSet, crate::CompileError> {
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
                            *label_offsets.get(lab).unwrap(),
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
                            *label_offsets.get(lab).unwrap(),
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
                            *label_offsets.get(lab).unwrap(),
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
                            *label_offsets.get(lab).unwrap(),
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
                            opcode:      AllocatedOpcode::DataSectionOffsetPlaceholder,
                            owning_span: None,
                            comment:     String::new(),
                        });
                    }
                    ControlFlowOp::LoadLabel(r1, ref lab) => {
                        let offset = *label_offsets.get(lab).unwrap();
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

        Ok(RealizedAbstractInstructionSet { ops: realized_ops })
    }

    fn resolve_labels(
        &mut self,
        data_section: &mut DataSection,
        iter_count: usize,
    ) -> Result<HashMap<Label, u64>, crate::CompileError> {
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

        if !remap_needed || !self.rewrite_far_jumps(&label_offsets) {
            // We didn't need to make any changes to the ops, so the labels are now correct.
            Ok(label_offsets)
        } else {
            // We did add new ops and so we need to update the label offsets.
            self.resolve_labels(data_section, iter_count + 1)
        }
    }

    fn map_label_offsets(&self, data_section: &DataSection) -> (bool, HashMap<Label, u64>) {
        let mut label_offsets = HashMap::new();
        let mut cur_offset = 0;

        // We decide here whether remapping the jumps _may_ be necessary.  We assume that if any
        // label is further than 18bits then we'll probably have to remap JNZIs, and we track JNEIs
        // specifically since they can only jump 12bits but are pretty rare.
        let mut furthest_offset = 0;
        let mut jnei_labels = HashSet::new();

        for op in &self.ops {
            match op.opcode {
                Either::Right(ControlFlowOp::Label(lab)) => {
                    label_offsets.insert(lab, cur_offset);
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
                // these ops will end up being exactly one op, so the cur_offset goes up one
                Either::Right(ControlFlowOp::Jump(..))
                | Either::Right(ControlFlowOp::JumpIfNotZero(..))
                | Either::Right(ControlFlowOp::Call(..))
                | Either::Right(ControlFlowOp::MoveAddress(..))
                | Either::Right(ControlFlowOp::LoadLabel(..))
                | Either::Left(_) => {
                    cur_offset += 1;
                }
                Either::Right(ControlFlowOp::JumpIfNotEq(_, _, lab)) => {
                    jnei_labels.insert(lab);
                    cur_offset += 1;
                }
                Either::Right(ControlFlowOp::Comment) => (),
                Either::Right(ControlFlowOp::DataSectionOffsetPlaceholder) => {
                    // If the placeholder is 32 bits, this is 1. if 64, this should be 2. We use LW
                    // to load the data, which loads a whole word, so for now this is 2.
                    cur_offset += 2
                }

                Either::Right(ControlFlowOp::PushAll(_))
                | Either::Right(ControlFlowOp::PopAll(_)) => unreachable!(
                    "fix me, pushall and popall don't really belong in control flow ops \
                        since they're not about control flow"
                ),
            }
        }

        let need_to_remap_jumps = furthest_offset > consts::EIGHTEEN_BITS
            || jnei_labels
                .iter()
                .any(|lab| label_offsets.get(lab).copied().unwrap() > consts::TWELVE_BITS);

        (need_to_remap_jumps, label_offsets)
    }

    /// If an instruction uses a label which can't fit in its immediate value then translate it
    /// into an instruction which loads the offset from the data section into a register and then
    /// uses the equivalent non-immediate instruction with the register.
    fn rewrite_far_jumps(&mut self, label_offsets: &HashMap<Label, u64>) -> bool {
        let min_ops = self.ops.len();
        let mut modified = false;

        self.ops = self
            .ops
            .drain(..)
            .fold(Vec::with_capacity(min_ops), |mut new_ops, op| {
                match &op.opcode {
                    Either::Right(ControlFlowOp::Jump(ref lab))
                    | Either::Right(ControlFlowOp::Call(ref lab)) => {
                        let offset = *label_offsets.get(lab).unwrap();
                        if offset <= consts::TWENTY_FOUR_BITS {
                            new_ops.push(op)
                        } else {
                            // Load the destination address into $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode:      Either::Right(ControlFlowOp::LoadLabel(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    *lab,
                                )),
                                comment:     String::new(),
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
                        let offset = *label_offsets.get(lab).unwrap();
                        if offset <= consts::TWELVE_BITS {
                            new_ops.push(op)
                        } else {
                            // Load the destination address into $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode:      Either::Right(ControlFlowOp::LoadLabel(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    *lab,
                                )),
                                comment:     String::new(),
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
                        let offset = *label_offsets.get(lab).unwrap();
                        if offset <= consts::EIGHTEEN_BITS {
                            new_ops.push(op)
                        } else {
                            // Load the destination address into $tmp.
                            new_ops.push(AllocatedAbstractOp {
                                opcode:      Either::Right(ControlFlowOp::LoadLabel(
                                    AllocatedRegister::Constant(ConstantRegister::Scratch),
                                    *lab,
                                )),
                                comment:     String::new(),
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
                        let offset = *label_offsets.get(lab).unwrap();
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
