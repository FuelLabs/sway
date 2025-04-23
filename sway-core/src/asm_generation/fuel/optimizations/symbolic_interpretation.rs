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

impl KnownRegValue {
    /// If the value can be represented as a constant, return it.
    fn register(&self) -> Option<VirtualRegister> {
        match self {
            KnownRegValue::Const(0) => Some(VirtualRegister::Constant(ConstantRegister::Zero)),
            KnownRegValue::Const(1) => Some(VirtualRegister::Constant(ConstantRegister::One)),
            KnownRegValue::Eq(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// If the value can be represented as a constant, return it.
    fn value(&self) -> Option<u64> {
        match self {
            KnownRegValue::Const(v) => Some(*v),
            KnownRegValue::Eq(VirtualRegister::Constant(ConstantRegister::Zero)) => Some(0),
            KnownRegValue::Eq(VirtualRegister::Constant(ConstantRegister::One)) => Some(1),
            KnownRegValue::Eq(_) => None,
        }
    }

    /// Check if the value depends on value of another register.
    fn depends_on(&self, reg: &VirtualRegister) -> bool {
        match self {
            KnownRegValue::Const(_) => false,
            KnownRegValue::Eq(v) => v == reg,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct KnownValues {
    values: FxHashMap<VirtualRegister, KnownRegValue>,
}

impl KnownValues {
    /// Resolve a register to a known value.
    fn resolve(&self, v: &VirtualRegister) -> Option<KnownRegValue> {
        match v {
            VirtualRegister::Constant(ConstantRegister::Zero) => Some(KnownRegValue::Const(0)),
            VirtualRegister::Constant(ConstantRegister::One) => Some(KnownRegValue::Const(1)),
            other => self.values.get(other).cloned(),
        }
    }

    /// Insert a known value for a register.
    fn assign(&mut self, dst: VirtualRegister, value: KnownRegValue) {
        self.values.retain(|_, v| !v.depends_on(&dst));
        self.values.insert(dst, value);
    }
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
                    known_values.retain(|_, v| !v.depends_on(d));
                }
                for d in op.def_const_registers() {
                    known_values.remove(d);
                    known_values.retain(|_, v| !v.depends_on(d));
                }
            }
        }
    }
}

impl AbstractInstructionSet {
    /// Symbolically interpret code and propagate known register values.
    pub(crate) fn interpret_propagate(mut self) -> AbstractInstructionSet {
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
        let mut known_values = KnownValues::default();

        for op in &mut self.ops {
            // Perform constant propagation on the instruction.
            let mut uses_regs: Vec<_> = op.use_registers_mut().into_iter().collect();
            for reg in uses_regs.iter_mut() {
                // We only optimize over virtual registers here, constant registers shouldn't be replaced
                if !reg.is_virtual() {
                    continue;
                }
                if let Some(r) = known_values.resolve(*reg).and_then(|r| r.register()) {
                    **reg = r;
                }
            }

            // Some instructions can be further simplified with the known values.
            match &mut op.opcode {
                // Conditional jumps can be simplified if we know the value of the register.
                Either::Right(ControlFlowOp::JumpIfNotZero(reg, lab)) => {
                    if let Some(con) = known_values.resolve(reg).and_then(|r| r.value()) {
                        if con == 0 {
                            op.opcode = Either::Left(VirtualOp::NOOP);
                        } else {
                            op.opcode = Either::Right(ControlFlowOp::Jump(lab.clone()));
                        }
                    }
                }
                _ => {}
            }

            // Some ops are known to produce certain results, interpret them here.
            let interpreted_op = match &op.opcode {
                Either::Left(VirtualOp::MOVI(dst, imm)) => {
                    known_values.assign(dst.clone(), KnownRegValue::Const(imm.value() as u64));
                    true
                }
                Either::Left(VirtualOp::MOVE(dst, src)) => {
                    if let Some(known) = known_values.resolve(src) {
                        known_values.assign(dst.clone(), known);
                    } else {
                        known_values.assign(dst.clone(), KnownRegValue::Eq(src.clone()));
                    }
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

                reset.apply(&op, &mut known_values.values);
            }
        }

        self
    }
}
