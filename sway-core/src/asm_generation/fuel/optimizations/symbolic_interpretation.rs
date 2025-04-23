//! Symbolic fuel-vm interpreter.

use either::Either;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::asm_lang::{ConstantRegister, ControlFlowOp, Label, Op, VirtualOp, VirtualRegister};

use super::super::abstract_instruction_set::AbstractInstructionSet;

#[derive(Clone, Debug, PartialEq, Eq)]
enum KnownRegValue {
    Const(u64),
    Eq(VirtualRegister),
}

/// What knowledge is lost after an op we don't know how to interpret?
#[derive(Clone, Debug)]
enum ResetKnown {
    /// Reset all known values
    Full,
    /// Reset non-virtual registers in addition to defs
    NonVirtual,
    /// Only the `def_registers` and `def_const_registers` are reset
    Defs,
}
impl ResetKnown {
    fn apply(&self, op: &Op, known_values: &mut FxHashMap<VirtualRegister, KnownRegValue>) {
        match self {
            ResetKnown::Full => {
                known_values.clear();
            }
            ResetKnown::NonVirtual => {
                Self::Defs.apply(op, known_values);
                known_values.retain(|k, _| {
                    if let VirtualRegister::Virtual(_) = k {
                        true
                    } else {
                        false
                    }
                });
            }
            ResetKnown::Defs => {
                for d in op.def_registers() {
                    known_values.remove(d);
                    known_values.retain(|_, v| KnownRegValue::Eq(d.clone()) != *v);
                }
                for d in op.def_const_registers() {
                    known_values.remove(d);
                    known_values.retain(|_, v| KnownRegValue::Eq(d.clone()) != *v);
                }
            }
        }
    }
}

impl AbstractInstructionSet {
    /// Remove redundant temporary variable registers.
    pub(crate) fn constant_register_propagation(mut self) -> AbstractInstructionSet {
        if self.ops.is_empty() {
            return self;
        }

        // The set of labels that are jump targets
        // todo: build proper control flow graph instead
        let jump_target_labels: FxHashSet<Label> = self
            .ops
            .iter()
            .filter_map(|op| match op.opcode {
                Either::Right(
                    ControlFlowOp::Jump(label)
                    | ControlFlowOp::JumpIfNotZero(_, label)
                    | ControlFlowOp::Call(label),
                ) => Some(label.clone()),
                _ => None,
            })
            .collect();

        // TODO: make this a struct with helper functions
        let mut known_values = FxHashMap::default();

        for op in &mut self.ops {
            // Perform constant propagation on the instruction.
            let mut uses_regs: Vec<_> = op.use_registers_mut().into_iter().collect();
            for reg in uses_regs.iter_mut() {
                // We only optimize over virtual registers here, constant registers shouldn't be replaced
                if !reg.is_virtual() {
                    continue;
                }
                let val: Option<&KnownRegValue> = known_values.get(*reg);

                match val {
                    Some(KnownRegValue::Eq(equivalent)) => {
                        **reg = equivalent.clone();
                    }
                    Some(KnownRegValue::Const(0)) => {
                        **reg = VirtualRegister::Constant(ConstantRegister::Zero);
                    }
                    Some(KnownRegValue::Const(1)) => {
                        **reg = VirtualRegister::Constant(ConstantRegister::One);
                    }
                    _ => {}
                }
            }

            // Some ops are known to produce certain results, interpret them here.
            let interpreted_op = match &op.opcode {
                Either::Left(VirtualOp::MOVI(dst, imm)) => {
                    known_values.insert(dst.clone(), KnownRegValue::Const(imm.value() as u64));
                    known_values.retain(|_, v| KnownRegValue::Eq(dst.clone()) != *v);
                    true
                }
                Either::Left(VirtualOp::MOVE(dst, src)) => {
                    if let Some(known) = known_values.get(src) {
                        known_values.insert(dst.clone(), known.clone());
                    } else {
                        known_values.insert(dst.clone(), KnownRegValue::Eq(src.clone()));
                    }
                    known_values.retain(|_, v| KnownRegValue::Eq(dst.clone()) != *v);
                    true
                }
                _ => false,
            };

            // If we don't know how to interpret the op, it's outputs are not known.
            if !interpreted_op {
                let reset = match &op.opcode {
                    Either::Left(op) => match op {
                        VirtualOp::ECAL(_, _, _, _) => ResetKnown::Full,
                        _ if op.has_side_effect() => ResetKnown::NonVirtual,
                        _ => ResetKnown::Defs,
                    },
                    Either::Right(op) => match op {
                        // If this is a jump target, then multiple jumps can reach it, and we can't
                        // assume to know register values.
                        ControlFlowOp::Label(label) => {
                            if jump_target_labels.contains(label) {
                                ResetKnown::Full
                            } else {
                                ResetKnown::Defs
                            }
                        }
                        // Jumping away doesn't invalidate state
                        ControlFlowOp::Jump(_) | ControlFlowOp::JumpIfNotZero(_, _) => {
                            ResetKnown::Defs
                        }
                        // todo: support call property. currently `def_const_registers`
                        //       doesn't contain return value, which seems incorrect
                        ControlFlowOp::Call(_) => ResetKnown::Full,
                        // These ops mark their outputs properly and cause no control-flow effects
                        ControlFlowOp::Comment
                        | ControlFlowOp::SaveRetAddr(_, _)
                        | ControlFlowOp::ConfigurablesOffsetPlaceholder
                        | ControlFlowOp::DataSectionOffsetPlaceholder
                        | ControlFlowOp::LoadLabel(_, _)
                        | ControlFlowOp::PushAll(_) => ResetKnown::Defs,
                        // This can be considered to destroy all known values
                        ControlFlowOp::PopAll(_) => ResetKnown::Full,
                    },
                };

                reset.apply(&op, &mut known_values);
            }
        }

        self
    }
}
