//! Symbolic fuel-vm interpreter.

use crate::asm_lang::VirtualImmediate18;
use either::Either;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use sway_types::Span;

use crate::asm_lang::{
    ConstantRegister, ControlFlowOp, JumpType, Label, Op, VirtualOp, VirtualRegister,
};

use super::super::abstract_instruction_set::AbstractInstructionSet;

/// A register value is known to contain the value of this expression.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Expr {
    Const(u64),
    Eq(VirtualRegister),
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
    Sub(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// If the value can be represented as a constant, return it.
    fn register(&self) -> Option<VirtualRegister> {
        match self {
            Expr::Const(0) => Some(VirtualRegister::Constant(ConstantRegister::Zero)),
            Expr::Const(1) => Some(VirtualRegister::Constant(ConstantRegister::One)),
            Expr::Eq(v) => Some(v.clone()),
            _ => None,
        }
    }

    /// If the value can be represented as a constant, return it.
    fn integer(&self) -> Option<u64> {
        match self {
            Expr::Const(v) => Some(*v),
            Expr::Eq(VirtualRegister::Constant(ConstantRegister::Zero)) => Some(0),
            Expr::Eq(VirtualRegister::Constant(ConstantRegister::One)) => Some(1),
            _ => None,
        }
    }

    /// Check if the value depends on value of another register.
    fn depends_on(&self, reg: &VirtualRegister) -> bool {
        match self {
            Expr::Const(_) => false,
            Expr::Eq(v) => v == reg,
            Expr::Add(vs) => vs.iter().any(|v| v.depends_on(reg)),
            Expr::Mul(vs) => vs.iter().any(|v| v.depends_on(reg)),
            Expr::Sub(l, r) => l.depends_on(reg) || r.depends_on(reg),
        }
    }

    /// Simplify the expression, if possible.
    fn simplify(self, ctx: &KnownValues) -> Self {
        match self {
            Expr::Eq(VirtualRegister::Constant(ConstantRegister::Zero)) => Self::Const(0),
            Expr::Eq(VirtualRegister::Constant(ConstantRegister::One)) => Self::Const(0),
            Expr::Eq(ref reg) => match ctx.resolve(reg) {
                Some(res) => res.simplify(ctx),
                None => self,
            },
            Expr::Add(vs) => {
                let mut simplified = Vec::new();
                for v in vs {
                    let v = v.simplify(ctx);
                    if v.integer() == Some(0) {
                        continue;
                    }
                    if let Expr::Add(vs) = v {
                        simplified.extend(vs);
                    } else {
                        simplified.push(v);
                    }
                }
                simplified.sort();

                let mut i = 0;
                while i + 1 < simplified.len() {
                    let lhs = simplified[i].integer();
                    let rhs = simplified[i + 1].integer();
                    if let (Some(l), Some(r)) = (lhs, rhs) {
                        if let Some(x) = l.checked_add(r) {
                            simplified[i] = Expr::Const(x);
                            simplified.remove(i + 1);
                        } else {
                            i += 1;
                        }
                    }

                    i += 1;
                }
                if simplified.len() == 0 {
                    Expr::Const(0)
                } else if simplified.len() == 1 {
                    simplified.pop().expect("Checked in if condition")
                } else {
                    Expr::Add(simplified)
                }
            }
            Expr::Mul(vs) => {
                let mut simplified = Vec::new();
                for v in vs {
                    let v = v.simplify(ctx);
                    if v.integer() == Some(1) {
                        continue;
                    }
                    if let Expr::Mul(vs) = v {
                        simplified.extend(vs);
                    } else {
                        simplified.push(v);
                    }
                }
                simplified.sort();

                let mut i = 0;
                while i + 1 < simplified.len() {
                    let lhs = simplified[i].integer();
                    let rhs = simplified[i + 1].integer();
                    if let (Some(l), Some(r)) = (lhs, rhs) {
                        if let Some(x) = l.checked_mul(r) {
                            simplified[i] = Expr::Const(x);
                            simplified.remove(i + 1);
                        } else {
                            i += 1;
                        }
                    }

                    i += 1;
                }
                if simplified.len() == 0 {
                    Expr::Const(1)
                } else if simplified.len() == 1 {
                    simplified.pop().expect("Checked in if condition")
                } else {
                    Expr::Mul(simplified)
                }
            }
            Expr::Sub(lhs, rhs) => {
                let lhs = lhs.simplify(&ctx);
                let rhs = rhs.simplify(&ctx);
                if lhs == rhs {
                    return Expr::Const(0);
                }
                match (lhs.integer(), rhs.integer()) {
                    (_, Some(0)) => lhs,
                    (Some(l), Some(r)) => match l.checked_sub(r) {
                        Some(v) => Expr::Const(v),
                        None => Expr::Sub(Box::new(lhs), Box::new(rhs)),
                    },
                    _ => Expr::Sub(Box::new(lhs), Box::new(rhs)),
                }
            }
            _ => self,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct KnownValues {
    /// Register values
    registers: FxHashMap<VirtualRegister, Expr>,
}

impl KnownValues {
    /// Resolve a register to a known value.
    fn resolve(&self, v: &VirtualRegister) -> Option<Expr> {
        match v {
            VirtualRegister::Constant(ConstantRegister::Zero) => Some(Expr::Const(0)),
            VirtualRegister::Constant(ConstantRegister::One) => Some(Expr::Const(1)),
            other => self.registers.get(other).cloned(),
        }
    }

    /// Clear values that depend on a register having a specific value.
    fn clear_dependent_on(&mut self, reg: &VirtualRegister) {
        self.registers.retain(|_, v| !v.depends_on(reg));
    }

    /// Insert a known value for a register.
    fn assign(&mut self, dst: VirtualRegister, value: Expr) {
        self.clear_dependent_on(&dst);
        self.registers.insert(dst, value);
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
    fn apply(&self, op: &Op, known_values: &mut KnownValues) {
        match self {
            ResetKnown::Full => {
                known_values.registers.clear();
            }
            ResetKnown::NonVirtual => {
                Self::Defs.apply(op, known_values);
                known_values
                    .registers
                    .retain(|k, _| matches!(k, VirtualRegister::Virtual(_)));
            }
            ResetKnown::Defs => {
                for d in op.def_registers() {
                    known_values.clear_dependent_on(d);
                    known_values.registers.remove(d);
                }
                for d in op.def_const_registers() {
                    known_values.clear_dependent_on(d);
                    known_values.registers.remove(d);
                }
            }
        }
    }
}

impl AbstractInstructionSet {
    /// Symbolically interpret code and propagate known register values.
    pub(crate) fn constant_propagate(mut self) -> AbstractInstructionSet {
        if self.ops.is_empty() {
            return self;
        }

        // The set of labels that are jump targets
        // todo: build proper control flow graph instead
        let mut jump_target_labels = FxHashMap::<Label, usize>::default();
        for op in &self.ops {
            if let Either::Right(ControlFlowOp::Jump { to, .. }) = &op.opcode {
                *jump_target_labels.entry(*to).or_default() += 1;
            }
        }

        let mut known_values = KnownValues::default();

        for op in &mut self.ops {
            // Perform constant propagation on the instruction.
            let mut uses_regs: Vec<_> = op.use_registers_mut().into_iter().collect();
            for reg in uses_regs.iter_mut() {
                // We only optimize over virtual registers here, constant registers shouldn't be replaced
                if !reg.is_virtual() {
                    continue;
                }
                if let Some(r) = known_values.resolve(reg).and_then(|r| r.register()) {
                    **reg = r;
                }
            }

            // Some instructions can be further simplified with the known values.
            match &mut op.opcode {
                // Conditional jumps can be simplified if we know the value of the register.
                Either::Right(ControlFlowOp::Jump {
                    to,
                    type_: JumpType::NotZero(reg),
                }) => {
                    if let Some(con) = known_values.resolve(reg).and_then(|r| r.integer()) {
                        if con == 0 {
                            let Entry::Occupied(mut count) = jump_target_labels.entry(*to) else {
                                unreachable!("Jump target label not found in jump_target_labels");
                            };
                            *count.get_mut() -= 1;
                            if *count.get() == 0 {
                                // Nobody jumps to this label anymore
                                jump_target_labels.remove(to);
                            }
                            op.opcode = Either::Left(VirtualOp::NOOP);
                        } else {
                            op.opcode = Either::Right(ControlFlowOp::Jump {
                                to: *to,
                                type_: JumpType::Unconditional,
                            });
                        }
                    }
                }
                _ => {}
            }

            // Some ops are known to produce certain results, interpret them here.
            let mut interpreted_op = match &op.opcode {
                Either::Left(VirtualOp::MOVE(dst, src)) => {
                    if let Some(known) = known_values.resolve(src) {
                        if known_values.resolve(dst) == Some(known.clone()) {
                            op.opcode = Either::Left(VirtualOp::NOOP);
                        } else {
                            known_values.assign(dst.clone(), known);
                        }
                    } else {
                        known_values.assign(dst.clone(), Expr::Eq(src.clone()));
                    }
                    true
                }
                Either::Left(VirtualOp::MOVI(dst, imm)) => {
                    let imm = Expr::Const(imm.value() as u64);
                    if known_values.resolve(dst) == Some(imm.clone()) {
                        op.opcode = Either::Left(VirtualOp::NOOP);
                    } else {
                        known_values.assign(dst.clone(), imm);
                    }
                    true
                }
                Either::Left(VirtualOp::ADD(dst, lhs, rhs)) => {
                    let lhs = Expr::Eq(lhs.clone());
                    let rhs = Expr::Eq(rhs.clone());
                    let expr = Expr::Add(vec![lhs, rhs]).simplify(&known_values);
                    if known_values.resolve(dst).as_ref() == Some(&expr) {
                        op.opcode = Either::Left(VirtualOp::NOOP);
                    } else {
                        known_values.assign(dst.clone(), expr);
                    }
                    true
                }
                Either::Left(VirtualOp::ADDI(dst, lhs, rhs)) => {
                    let lhs = Expr::Eq(lhs.clone());
                    let rhs = Expr::Const(rhs.value() as u64);
                    let expr = Expr::Add(vec![lhs, rhs]).simplify(&known_values);
                    if known_values.resolve(dst).as_ref() == Some(&expr) {
                        op.opcode = Either::Left(VirtualOp::NOOP);
                    } else {
                        known_values.assign(dst.clone(), expr);
                    }
                    true
                }
                Either::Left(VirtualOp::SUB(dst, lhs, rhs)) => {
                    let lhs = Expr::Eq(lhs.clone());
                    let rhs = Expr::Eq(rhs.clone());
                    let expr = Expr::Sub(Box::new(lhs), Box::new(rhs)).simplify(&known_values);
                    if known_values.resolve(dst).as_ref() == Some(&expr) {
                        op.opcode = Either::Left(VirtualOp::NOOP);
                    } else {
                        known_values.assign(dst.clone(), expr);
                    }
                    true
                }
                Either::Left(VirtualOp::SUBI(dst, lhs, rhs)) => {
                    let lhs = Expr::Eq(lhs.clone());
                    let rhs = Expr::Const(rhs.value() as u64);
                    let expr = Expr::Sub(Box::new(lhs), Box::new(rhs)).simplify(&known_values);
                    if known_values.resolve(dst).as_ref() == Some(&expr) {
                        op.opcode = Either::Left(VirtualOp::NOOP);
                    } else {
                        known_values.assign(dst.clone(), expr);
                    }
                    true
                }
                Either::Left(VirtualOp::MUL(dst, lhs, rhs)) => {
                    let lhs = Expr::Eq(lhs.clone());
                    let rhs = Expr::Eq(rhs.clone());
                    let expr = Expr::Mul(vec![lhs, rhs]).simplify(&known_values);
                    if known_values.resolve(dst).as_ref() == Some(&expr) {
                        op.opcode = Either::Left(VirtualOp::NOOP);
                    } else {
                        known_values.assign(dst.clone(), expr);
                    }
                    true
                }
                Either::Left(VirtualOp::MULI(dst, lhs, rhs)) => {
                    let lhs = Expr::Eq(lhs.clone());
                    let rhs = Expr::Const(rhs.value() as u64);
                    let expr = Expr::Mul(vec![lhs, rhs]).simplify(&known_values);
                    if known_values.resolve(dst).as_ref() == Some(&expr) {
                        op.opcode = Either::Left(VirtualOp::NOOP);
                    } else {
                        known_values.assign(dst.clone(), expr);
                    }
                    true
                }
                _ => false,
            };

            // If the final value can be set directly, do so.
            if let Either::Left(op) = &mut op.opcode {
                if !op.has_side_effect() {
                    let defs = op.def_registers();
                    if defs.len() == 1 {
                        let def = defs.first().expect("len == 1 checked above");
                        if let Some(known) = known_values.resolve(def) {
                            if let Some(v) = known.integer() {
                                if let Ok(imm) = VirtualImmediate18::new(v, Span::dummy()) {
                                    *op = VirtualOp::MOVI((*def).clone(), imm);
                                    interpreted_op = true;
                                }
                            }
                        }
                    }
                }
            }

            // If we don't know how to interpret the op, it's outputs are not known.
            if !interpreted_op {
                let reset = match &op.opcode {
                    Either::Left(op) => match op {
                        // These always require a full reset
                        VirtualOp::ECAL(_, _, _, _) => ResetKnown::Full,
                        // These ops are not known have register-related side effects
                        VirtualOp::GT(_, _, _)
                        | VirtualOp::GTF(_, _, _)
                        | VirtualOp::MCP(_, _, _)
                        | VirtualOp::MCPI(_, _, _)
                        | VirtualOp::LB(_, _, _)
                        | VirtualOp::LW(_, _, _)
                        | VirtualOp::SB(_, _, _)
                        | VirtualOp::SW(_, _, _) => ResetKnown::Defs,
                        // TODO: this constraint can be relaxed
                        _ if op.has_side_effect() => ResetKnown::Full,
                        _ => ResetKnown::Defs,
                    },
                    Either::Right(op) => match op {
                        // If this is a jump target, then multiple jumps can reach it, and we can't
                        // assume to know register values.
                        ControlFlowOp::Label(label) => {
                            if jump_target_labels.contains_key(label) {
                                ResetKnown::Full
                            } else {
                                ResetKnown::Defs
                            }
                        }
                        // Jumping away doesn't invalidate state, but for calls:
                        // TODO: `def_const_registers` doesn't contain return value, which
                        //       seems incorrect, so I'm clearing everything as a precaution
                        ControlFlowOp::Jump { type_, .. } => match type_ {
                            JumpType::Call => ResetKnown::Full,
                            _ => ResetKnown::Defs,
                        },
                        // These ops mark their outputs properly and cause no control-flow effects
                        ControlFlowOp::Comment
                        | ControlFlowOp::SaveRetAddr(_, _)
                        | ControlFlowOp::ConfigurablesOffsetPlaceholder
                        | ControlFlowOp::DataSectionOffsetPlaceholder => ResetKnown::Defs,
                        // This changes the stack pointer
                        ControlFlowOp::PushAll(_) => ResetKnown::NonVirtual,
                        // This can be considered to destroy all known values
                        ControlFlowOp::PopAll(_) => ResetKnown::Full,
                    },
                };

                reset.apply(op, &mut known_values);
            }
        }

        self
    }
}
