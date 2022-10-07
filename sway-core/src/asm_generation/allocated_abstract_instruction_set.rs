use crate::asm_lang::{
    allocated_ops::{AllocatedOpcode, AllocatedRegister},
    AllocatedAbstractOp, ConstantRegister, ControlFlowOp, Label, RealizedOp, VirtualImmediate12,
    VirtualImmediate18, VirtualImmediate24,
};

use super::{DataSection, RealizedAbstractInstructionSet};

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

    /// Runs two passes -- one to get the instruction offsets of the labels
    /// and one to replace the labels in the organizational ops
    pub(crate) fn realize_labels(
        self,
        data_section: &DataSection,
    ) -> RealizedAbstractInstructionSet {
        let mut label_namespace: HashMap<&Label, u64> = Default::default();
        let mut offset_map = vec![];
        let mut counter = 0;
        for op in &self.ops {
            offset_map.push(counter);
            match op.opcode {
                Either::Right(ControlFlowOp::Label(ref lab)) => {
                    label_namespace.insert(lab, counter);
                }
                // A special case for LWDataId which may be 1 or 2 ops, depending on the source size.
                Either::Left(AllocatedOpcode::LWDataId(_, ref data_id)) => {
                    let has_copy_type = data_section.has_copy_type(data_id).expect(
                        "Internal miscalculation in data section -- \
                        data id did not match up to any actual data",
                    );
                    counter += if has_copy_type { 1 } else { 2 };
                }
                // these ops will end up being exactly one op, so the counter goes up one
                Either::Right(ControlFlowOp::Jump(..))
                | Either::Right(ControlFlowOp::JumpIfNotEq(..))
                | Either::Right(ControlFlowOp::JumpIfNotZero(..))
                | Either::Right(ControlFlowOp::Call(..))
                | Either::Right(ControlFlowOp::MoveAddress(..))
                | Either::Left(_) => {
                    counter += 1;
                }
                Either::Right(ControlFlowOp::Comment) => (),
                Either::Right(ControlFlowOp::DataSectionOffsetPlaceholder) => {
                    // If the placeholder is 32 bits, this is 1. if 64, this should be 2. We use LW
                    // to load the data, which loads a whole word, so for now this is 2.
                    counter += 2
                }

                Either::Right(ControlFlowOp::PushAll(_))
                | Either::Right(ControlFlowOp::PopAll(_)) => unreachable!(
                    "fix me, pushall and popall don't really belong in control flow ops \
                        since they're not about control flow"
                ),
            }
        }

        let mut realized_ops = vec![];
        for (
            ix,
            AllocatedAbstractOp {
                opcode,
                comment,
                owning_span,
            },
        ) in self.ops.clone().into_iter().enumerate()
        {
            let offset = offset_map[ix];
            match opcode {
                Either::Left(op) => realized_ops.push(RealizedOp {
                    opcode: op,
                    owning_span,
                    comment,
                    offset,
                }),
                Either::Right(org_op) => match org_op {
                    ControlFlowOp::Jump(ref lab) | ControlFlowOp::Call(ref lab) => {
                        let imm = VirtualImmediate24::new_unchecked(
                            *label_namespace.get(lab).unwrap(),
                            "Programs with more than 2^24 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::JI(imm),
                            owning_span,
                            comment,
                            offset,
                        });
                    }
                    ControlFlowOp::JumpIfNotEq(r1, r2, ref lab) => {
                        let imm = VirtualImmediate12::new_unchecked(
                            *label_namespace.get(lab).unwrap(),
                            "Programs with more than 2^12 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::JNEI(r1, r2, imm),
                            owning_span,
                            comment,
                            offset,
                        });
                    }
                    ControlFlowOp::JumpIfNotZero(r1, ref lab) => {
                        let imm = VirtualImmediate18::new_unchecked(
                            *label_namespace.get(lab).unwrap(),
                            "Programs with more than 2^18 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::JNZI(r1, imm),
                            owning_span,
                            comment,
                            offset,
                        });
                    }
                    ControlFlowOp::MoveAddress(r1, ref lab) => {
                        let imm = VirtualImmediate18::new_unchecked(
                            *label_namespace.get(lab).unwrap(),
                            "Programs with more than 2^18 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::MOVI(r1, imm),
                            owning_span,
                            comment,
                            offset,
                        });
                    }
                    ControlFlowOp::DataSectionOffsetPlaceholder => {
                        realized_ops.push(RealizedOp {
                            opcode: AllocatedOpcode::DataSectionOffsetPlaceholder,
                            owning_span: None,
                            comment: String::new(),
                            offset,
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
        RealizedAbstractInstructionSet { ops: realized_ops }
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
