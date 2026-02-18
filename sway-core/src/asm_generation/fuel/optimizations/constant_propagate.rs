use super::super::abstract_instruction_set::AbstractInstructionSet;
use crate::asm_lang::{
    ConstantRegister, ControlFlowOp, JumpType, Label, Op, VirtualImmediate18, VirtualOp,
    VirtualRegister,
};
use either::Either;
use rustc_hash::FxHashMap;
use std::{
    collections::hash_map::Entry,
    ops::{BitAnd, BitOr, BitXor},
};
use sway_types::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
enum KnownRegValue {
    Const(u64),
    Eq(VirtualRegister),
}

impl KnownRegValue {
    /// If the value can be represented as a register, return it.
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

    fn value_as_imm18(&self) -> Option<VirtualImmediate18> {
        let raw = self.value()?;
        VirtualImmediate18::try_new(raw, Span::dummy()).ok()
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

    /// Clear values that depend on a register having a specific value.
    fn clear_dependent_on(&mut self, reg: &VirtualRegister) {
        let mut q = vec![reg.clone()];

        while let Some(reg) = q.pop() {
            let keys = self
                .values
                .extract_if(|_, v| v.depends_on(&reg))
                .map(|(k, _)| k)
                .collect::<Vec<_>>();
            q.extend(keys);

            self.values.remove(&reg);
        }
    }

    /// Insert a known value for a register.
    fn assign(&mut self, dst: VirtualRegister, value: KnownRegValue) {
        self.clear_dependent_on(&dst);
        self.values.insert(dst, value);
    }
}

/// What knowledge is lost after an op we don't know how to interpret?
#[derive(Clone, Debug)]
enum ResetKnown {
    /// Do nothing
    Nothing,
    /// Only the `def_registers` and `def_const_registers` are reset
    Defs,
    /// Reset non-virtual registers in addition to defs
    DefsAndNonVirtuals,
    /// Reset all known values
    Full,
}

impl ResetKnown {
    fn apply(&self, op: &Op, known_values: &mut KnownValues) {
        match self {
            ResetKnown::Nothing => {}
            ResetKnown::Defs => {
                for d in op.def_registers() {
                    known_values.clear_dependent_on(d);
                    known_values.values.remove(d);
                }
                for d in op.def_const_registers() {
                    known_values.clear_dependent_on(d);
                    known_values.values.remove(d);
                }
            }
            ResetKnown::DefsAndNonVirtuals => {
                Self::Defs.apply(op, known_values);
                known_values
                    .values
                    .retain(|k, _| matches!(k, VirtualRegister::Virtual(_)));
            }
            ResetKnown::Full => {
                known_values.values.clear();
            }
        }
    }
}

impl AbstractInstructionSet {
    /// Symbolically interpret code and propagate known register values.
    pub(crate) fn constant_propagate(
        mut self,
        mut log: impl FnMut(&str),
    ) -> AbstractInstructionSet {
        if self.ops.is_empty() {
            return self;
        }

        // The set of labels that are jump targets, and how many places jump to them.
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
            for reg in op.use_registers_mut() {
                // We only optimize over virtual registers here, constant registers shouldn't be replaced
                if !reg.is_virtual() {
                    continue;
                }

                if let Some(r) = known_values.resolve(reg).and_then(|r| r.register()) {
                    *reg = r;
                }
            }

            // Replace "JNZ reg LABEL" to
            // - NOOP if the reg is zero, or
            // - "JMP LABEL" if reg is zero
            if let Either::Right(ControlFlowOp::Jump {
                to,
                type_: JumpType::NotZero(reg),
            }) = &mut op.opcode
            {
                if let Some(con) = known_values.resolve(reg).and_then(|r| r.value()) {
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

            // This macro must be declared here to be able to capture some variables
            macro_rules! transform_operator {
                (assign_value; $dst:ident, $l:ident, $r:ident; left) => {
                    known_values.assign($dst.clone(), KnownRegValue::Eq($l.clone()));
                };
                (assign_value; $dst:ident, $l:ident, $r:ident; right) => {
                    known_values.assign($dst.clone(), KnownRegValue::Eq($r.clone()));
                };
                (assign_value; $dst:ident, $l:ident, $r:ident; $v:literal) => {
                    known_values.assign($dst.clone(), KnownRegValue::Const($v));
                };
                // If the value is one of the operands, we will use MOVE
                // if is a literal we will use MOVI
                (new_opcode; $dst:ident, $l:ident, $r:ident; left) => {
                    Either::Left(VirtualOp::MOVE($dst.clone(), $l.clone()))
                };
                (new_opcode; $dst:ident, $l:ident, $r:ident; right) => {
                    Either::Left(VirtualOp::MOVE($dst.clone(), $r.clone()))
                };
                (new_opcode; $dst:ident, $l:ident, $r:ident; $v:literal) => {{
                    let imm = VirtualImmediate18::try_new($v, Span::dummy()).ok()?;
                    Either::Left(VirtualOp::MOVI($dst.clone(), imm))
                }};
                // if left == $initial_value assigns $end_value
                (gen; $op:ident, $opI:ident; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident; if left is $initial_value:literal assigns $end_value:tt; $($rest:tt)*) => {
                    if let (Some(KnownRegValue::Const($initial_value)), _) = (&$lv, &$rv) {
                        let new_opcode = transform_operator!{new_opcode; $dst, $l, $r; $end_value};
                        // The line above can bail so we must run it before we change anything
                        transform_operator!{assign_value; $dst, $l, $r; $end_value};
                        op.opcode = new_opcode;
                        return Some(ResetKnown::Nothing);
                    }
                    transform_operator!{gen; $op, $opI; $dst, $l, $r, $lv, $rv; $($rest)*}
                };
                // if right == $initial_value assigns $end_value
                (gen; $op:ident, $opI:ident; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident; if right is $initial_value:literal assigns $end_value:tt; $($rest:tt)*) => {
                    if let (_, Some(KnownRegValue::Const($initial_value))) = (&$lv, &$rv) {
                        let new_opcode = transform_operator!{new_opcode; $dst, $l, $r; $end_value};
                        // The line above can bail so we must run it before we change anything
                        transform_operator!{assign_value; $dst, $l, $r; $end_value};
                        op.opcode = new_opcode;
                        return Some(ResetKnown::Nothing);
                    }
                    transform_operator!{gen; $op, $opI; $dst, $l, $r, $lv, $rv; $($rest)*}
                };
                // When both register values are known transform to MOVI
                (gen; $op:ident, $opI:ident; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident; both_known: $f:path; $($rest:tt)*) => {
                    if let (Some(KnownRegValue::Const($lv)), Some(KnownRegValue::Const($rv))) = (&$lv, &$rv) {
                        let raw = $f(*$lv, (*$rv).try_into().ok()?)?.into();
                        let imm = VirtualImmediate18::try_new(raw, Span::dummy()).ok()?;
                        known_values.assign($dst.clone(), KnownRegValue::Const(raw));
                        op.opcode = Either::Left(VirtualOp::MOVI($dst.clone(), imm));
                        return Some(ResetKnown::Nothing);
                    }
                    transform_operator!{gen; $op, $opI; $dst, $l, $r, $lv, $rv; $($rest)*}
                };
                // Transform from $op to $opI because $op is commutative
                (gen; $op:ident, $opI:ident; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident; commutative: true; $($rest:tt)*) => {
                    if let (Some(KnownRegValue::Const($lv)), _) = (&$lv, &$rv) {
                        op.opcode = Either::Left(VirtualOp::$opI($dst.clone(), $r.clone(), (*$lv).try_into().ok()?));
                        return Some(ResetKnown::Defs);
                    }
                    transform_operator!{gen; $op, $opI; $dst, $l, $r, $lv, $rv; $($rest)*}
                };
                (gen; $op:ident, $opI:ident; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident;) => {
                };
                (gen; $op:ident, $opI:ident; $($rest:tt)*) => {
                    compile_error!(stringify!($($rest)*))
                };
                // Transform from $op to $opI if $opI was defined.
                // Otherwise does not generate code
                (replace_imm; None; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident) => {
                };
                (replace_imm; $opI:ident; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident) => {
                    if let (_, Some(KnownRegValue::Const(rv))) = (&$lv, &$rv) {
                        op.opcode = Either::Left(VirtualOp::$opI($dst.clone(), $l.clone(), (*rv).try_into().ok()?));
                        return Some(ResetKnown::Defs);
                    }
                };
                ($op:ident, $opI:ident; $($rest:tt)*) => {{
                    let mut f = || -> Option<ResetKnown> {
                        match &op.opcode {
                            Either::Left(VirtualOp::$op(dst, l, r)) => {
                                let lv = known_values.resolve(&l);
                                let rv = known_values.resolve(&r);
                                log(&format!("    {:?} {:?}\n", lv, rv));

                                transform_operator!{gen; $op, $opI; dst, l, r, lv, rv; $($rest)*}
                                transform_operator!{replace_imm; $opI; dst, l, r, lv, rv}
                            }
                            _ => {}
                        };
                        None
                    };
                    f()
                }};
            }

            let before = format!("{op}");
            log(&format!("{op}\n"));

            // Propagate constant of some ops
            // Also transform them if they registers are known
            let reset = match op.opcode.clone() {
                Either::Left(VirtualOp::MOVI(dst, imm)) => {
                    let imm = KnownRegValue::Const(imm.value() as u64);

                    // transform
                    // movi <dest is known> <imm>
                    // to
                    // noop
                    if known_values.resolve(&dst) == Some(imm.clone()) {
                        op.opcode = Either::Left(VirtualOp::NOOP);
                    } else {
                        known_values.assign(dst, imm);
                    }

                    Some(ResetKnown::Nothing)
                }
                Either::Left(VirtualOp::MOVE(dst, src)) => {
                    if let Some(known_src) = known_values.resolve(&src) {
                        // transform
                        // move <dest is known> <src is known>
                        // to
                        // noop
                        if known_values.resolve(&dst) == Some(known_src.clone()) {
                            op.opcode = Either::Left(VirtualOp::NOOP);
                        } else {
                            // transform
                            // move <dest is unknown> <src is known>
                            // to
                            // movi dest imm
                            if let Some(imm) = known_src.value_as_imm18() {
                                op.opcode = Either::Left(VirtualOp::MOVI(dst.clone(), imm));
                            }

                            known_values.assign(dst.clone(), known_src);
                        }
                    } else {
                        known_values.assign(dst, KnownRegValue::Eq(src));
                    }

                    Some(ResetKnown::Nothing)
                }

                // algebraic operators
                Either::Left(VirtualOp::ADD(..)) => transform_operator! {ADD, ADDI;
                    both_known: u64::checked_add;
                    if left is 0 assigns right;
                    if right is 0 assigns left;
                    commutative: true;
                },
                Either::Left(VirtualOp::SUB(..)) => transform_operator! {SUB, SUBI;
                    both_known: u64::checked_sub;
                    if right is 0 assigns left;
                },
                Either::Left(VirtualOp::MUL(..)) => transform_operator! {MUL, MULI;
                    both_known: u64::checked_mul;
                    if left is 1 assigns right;
                    if right is 1 assigns left;
                    if left is 0 assigns 0;
                    if right is 0 assigns 0;
                    commutative: true;
                },
                Either::Left(VirtualOp::DIV(..)) => transform_operator! {DIV, DIVI;
                    both_known: u64::checked_div;
                    if right is 1 assigns left;
                    if left is 0 assigns 0;
                },
                Either::Left(VirtualOp::EXP(..)) => transform_operator! {EXP, EXPI;
                    both_known: u64::checked_pow;
                    if right is 0 assigns 1;
                    if right is 1 assigns left;
                    if left is 0 assigns 0;
                    if left is 1 assigns 1;
                },
                Either::Left(VirtualOp::MLOG(..)) => transform_operator! {MLOG, None;
                    both_known: u64::checked_ilog;
                },
                Either::Left(VirtualOp::MOD(..)) => transform_operator! {MOD, MODI;
                    both_known: u64::checked_rem;
                    if right is 1 assigns 0;
                },
                Either::Left(VirtualOp::MROO(..)) => transform_operator! {MROO, None;
                    both_known: checked_nth_root;
                    if right is 1 assigns left;
                },

                // bitwise
                Either::Left(VirtualOp::AND(..)) => transform_operator! {AND, ANDI;
                    both_known: u64_bitand;
                    if left is 0 assigns 0;
                    if right is 0 assigns 0;
                     commutative: true;
                },
                Either::Left(VirtualOp::OR(..)) => transform_operator! {OR, ORI;
                    both_known: u64_bitor;
                    if left is 0 assigns right;
                    if right is 0 assigns left;
                    commutative: true;
                },
                Either::Left(VirtualOp::XOR(..)) => transform_operator! {XOR, XORI;
                    both_known: u64_bitxor;
                    if left is 0 assigns right;
                    if right is 0 assigns left;
                    commutative: true;
                },
                Either::Left(VirtualOp::SLL(..)) => transform_operator! {SLL, SLLI;
                    both_known: u64::checked_shl;
                    if right is 0 assigns left;
                },
                Either::Left(VirtualOp::SRL(..)) => transform_operator! {SRL, SRLI;
                    both_known: u64::checked_shr;
                    if right is 0 assigns left;
                },

                // Comparisons
                Either::Left(VirtualOp::EQ(..)) => transform_operator! {EQ, None;
                    both_known: u64_eq;
                },
                Either::Left(VirtualOp::GT(..)) => transform_operator! {GT, None;
                    both_known: u64_gt;
                },
                Either::Left(VirtualOp::LT(..)) => transform_operator! {LT, None;
                    both_known: u64_lt;
                },
                _ => None,
            };

            let after = format!("{op}");
            if before != after {
                log(&format!("    changed to: {op}\n"));
            }

            let reset = match reset {
                None => {
                    match &op.opcode {
                        Either::Left(op) => match op {
                            VirtualOp::ECAL(_, _, _, _) => ResetKnown::Full,
                            // TODO: this constraint can be relaxed
                            _ if op.has_side_effect() => ResetKnown::Full,
                            _ => ResetKnown::Defs,
                        },
                        Either::Right(op) => match op {
                            // If this is a jump target, then multiple execution paths can lead to it,
                            // and we can't assume to know register values.
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
                            | ControlFlowOp::ConfigurablesOffsetPlaceholder
                            | ControlFlowOp::DataSectionOffsetPlaceholder => ResetKnown::Defs,
                            // This changes the stack pointer
                            ControlFlowOp::PushAll(_) => ResetKnown::DefsAndNonVirtuals,
                            // This can be considered to destroy all known values
                            ControlFlowOp::PopAll(_) => ResetKnown::Full,
                        },
                    }
                }
                Some(reset) => reset,
            };

            log(&format!("    {reset:?}\n"));
            reset.apply(op, &mut known_values);
        }

        self
    }
}

#[inline(always)]
fn u64_bitand(l: u64, r: u64) -> Option<u64> {
    Some(l.bitand(r))
}

#[inline(always)]
fn u64_bitor(l: u64, r: u64) -> Option<u64> {
    Some(l.bitor(r))
}

#[inline(always)]
fn u64_bitxor(l: u64, r: u64) -> Option<u64> {
    Some(l.bitxor(r))
}

#[inline(always)]
fn u64_eq(l: u64, r: u64) -> Option<u64> {
    Some(if l == r { 1 } else { 0 })
}

#[inline(always)]
fn u64_gt(l: u64, r: u64) -> Option<u64> {
    Some(if l > r { 1 } else { 0 })
}

#[inline(always)]
fn u64_lt(l: u64, r: u64) -> Option<u64> {
    Some(if l < r { 1 } else { 0 })
}

// Copied from fuel_vm to guarantee exactly same behaviour.
// See fuel-vm/src/interpreter/executors/instruction.rs
pub fn checked_nth_root(target: u64, nth_root: u64) -> Option<u64> {
    if nth_root == 0 {
        // Zeroth root is not defined
        return None;
    }

    if nth_root == 1 || target <= 1 {
        // Corner cases
        return Some(target);
    }

    if nth_root >= target || nth_root > 64 {
        // For any root >= target, result always 1
        // For any n>1, n**64 can never fit into u64
        return Some(1);
    }

    let nth_root = u32::try_from(nth_root).expect("Never loses bits, checked above");

    // Use floating point operation to get an approximation for the starting point.
    // This is at most off by one in either direction.

    let powf = f64::powf;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let guess = powf(target as f64, (nth_root as f64).recip()) as u64;

    debug_assert!(guess != 0, "This should never occur for {{target, n}} > 1");

    // Check if a value raised to nth_power is below the target value, handling overflow
    // correctly
    let is_nth_power_below_target = |v: u64| match v.checked_pow(nth_root) {
        Some(pow) => target < pow,
        None => true, // v**nth_root >= 2**64 and target < 2**64
    };

    // Compute guess**n to check if the guess is too large.
    // Note that if guess == 1, then g1 == 1 as well, meaning that we will not return
    // here.
    if is_nth_power_below_target(guess) {
        return Some(guess.saturating_sub(1));
    }

    // Check if the initial guess was correct
    let guess_plus_one = guess
        .checked_add(1)
        .expect("Guess cannot be u64::MAX, as we have taken a root > 2 of a value to get it");
    if is_nth_power_below_target(guess_plus_one) {
        return Some(guess);
    }

    // If not, then the value above must be the correct one.
    Some(guess_plus_one)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    fn optimise(
        ops: impl IntoIterator<Item = Op>,
        f: impl FnOnce(AbstractInstructionSet) -> AbstractInstructionSet,
    ) {
        let mut ops = AbstractInstructionSet {
            function: None,
            ops: ops.into_iter().collect(),
        };

        for i in 0..ops.ops.len() {
            ops.ops[i].comment = i.to_string();
        }

        f(ops);
    }

    #[test]
    fn constant_propagate_transform() {
        let mut str = String::new();
        let capture = |s: &str| {
            str.push_str(s);
        };
        optimise(
            [
                VirtualOp::movi("0", 10).into(),
                VirtualOp::movi("1", 10).into(),
                VirtualOp::movi("2", 2).into(),
                VirtualOp::movi("3", 8).into(),
                VirtualOp::movi("4", 9).into(),
                VirtualOp::movi("0", 10).into(),
                VirtualOp::r#move("0", "1").into(),
                //add
                VirtualOp::add(ConstantRegister::FuncArg0, "0", "1").into(),
                VirtualOp::add(ConstantRegister::FuncArg1, ConstantRegister::Zero, "0").into(),
                VirtualOp::add(ConstantRegister::FuncArg2, "0", ConstantRegister::Zero).into(),
                VirtualOp::add(ConstantRegister::FuncArg3, "5", "0").into(),
                VirtualOp::add(ConstantRegister::FuncArg4, "0", "5").into(),
                //sub
                VirtualOp::sub(ConstantRegister::FuncArg0, "0", "1").into(),
                VirtualOp::sub(ConstantRegister::FuncArg1, "0", ConstantRegister::Zero).into(),
                //mul
                VirtualOp::mul(ConstantRegister::FuncArg0, "0", "1").into(),
                VirtualOp::mul(ConstantRegister::FuncArg1, ConstantRegister::One, "0").into(),
                VirtualOp::mul(ConstantRegister::FuncArg2, "0", ConstantRegister::One).into(),
                VirtualOp::mul(ConstantRegister::FuncArg3, ConstantRegister::Zero, "0").into(),
                VirtualOp::mul(ConstantRegister::FuncArg4, "0", ConstantRegister::Zero).into(),
                VirtualOp::mul(ConstantRegister::FuncArg5, "5", "0").into(),
                VirtualOp::mul(ConstantRegister::FuncArg0, "0", "5").into(),
                //div
                VirtualOp::div(ConstantRegister::FuncArg0, "0", "1").into(),
                VirtualOp::div(ConstantRegister::FuncArg1, "0", ConstantRegister::One).into(),
                VirtualOp::div(ConstantRegister::FuncArg2, ConstantRegister::Zero, "0").into(),
                VirtualOp::div(
                    ConstantRegister::FuncArg2,
                    ConstantRegister::Zero,
                    ConstantRegister::Zero,
                )
                .into(),
                //exp
                VirtualOp::exp(ConstantRegister::FuncArg0, "0", "2").into(),
                VirtualOp::exp(
                    ConstantRegister::FuncArg0,
                    ConstantRegister::Zero,
                    ConstantRegister::Zero,
                )
                .into(),
                VirtualOp::exp(ConstantRegister::FuncArg1, "0", ConstantRegister::Zero).into(),
                VirtualOp::exp(ConstantRegister::FuncArg2, "0", ConstantRegister::One).into(),
                VirtualOp::exp(ConstantRegister::FuncArg3, ConstantRegister::Zero, "0").into(),
                VirtualOp::exp(ConstantRegister::FuncArg4, ConstantRegister::One, "0").into(),
                //mlog
                VirtualOp::mlog(ConstantRegister::FuncArg0, "3", "2").into(),
                //mod
                VirtualOp::r#mod(ConstantRegister::FuncArg0, "4", "2").into(),
                VirtualOp::r#mod(ConstantRegister::FuncArg1, "4", ConstantRegister::One).into(),
                //mroo
                VirtualOp::mroo(ConstantRegister::FuncArg0, "4", "2").into(),
                VirtualOp::mroo(ConstantRegister::FuncArg1, "4", ConstantRegister::One).into(),
                //and
                VirtualOp::and(ConstantRegister::FuncArg0, "0", "2").into(),
                VirtualOp::and(ConstantRegister::FuncArg1, ConstantRegister::Zero, "2").into(),
                VirtualOp::and(ConstantRegister::FuncArg2, "2", ConstantRegister::Zero).into(),
                VirtualOp::and(ConstantRegister::FuncArg3, "5", "0").into(),
                VirtualOp::and(ConstantRegister::FuncArg4, "0", "5").into(),
                //or
                VirtualOp::or(ConstantRegister::FuncArg0, "0", "2").into(),
                VirtualOp::or(ConstantRegister::FuncArg1, ConstantRegister::Zero, "2").into(),
                VirtualOp::or(ConstantRegister::FuncArg2, "2", ConstantRegister::Zero).into(),
                VirtualOp::or(ConstantRegister::FuncArg3, "5", "0").into(),
                VirtualOp::or(ConstantRegister::FuncArg4, "0", "5").into(),
                //xor
                VirtualOp::xor(ConstantRegister::FuncArg0, "0", "2").into(),
                VirtualOp::xor(ConstantRegister::FuncArg1, ConstantRegister::Zero, "2").into(),
                VirtualOp::xor(ConstantRegister::FuncArg2, "2", ConstantRegister::Zero).into(),
                VirtualOp::xor(ConstantRegister::FuncArg3, "5", "0").into(),
                VirtualOp::xor(ConstantRegister::FuncArg4, "0", "5").into(),
                //sll
                VirtualOp::sll(ConstantRegister::FuncArg0, "0", "2").into(),
                VirtualOp::sll(ConstantRegister::FuncArg2, "2", ConstantRegister::Zero).into(),
                //srl
                VirtualOp::srl(ConstantRegister::FuncArg0, "0", "2").into(),
                VirtualOp::srl(ConstantRegister::FuncArg2, "2", ConstantRegister::Zero).into(),
                //eq
                VirtualOp::eq(ConstantRegister::FuncArg0, "0", "2").into(),
                //gt
                VirtualOp::gt(ConstantRegister::FuncArg0, "0", "2").into(),
                //lt
                VirtualOp::lt(ConstantRegister::FuncArg0, "0", "2").into(),
            ],
            |ops| ops.constant_propagate(capture),
        );

        expect![[r#"
            movi $r0 i10                            ; 0
                Nothing
            movi $r1 i10                            ; 1
                Nothing
            movi $r2 i2                             ; 2
                Nothing
            movi $r3 i8                             ; 3
                Nothing
            movi $r4 i9                             ; 4
                Nothing
            movi $r0 i10                            ; 5
                changed to: noop                                    ; 5
                Nothing
            move $r0 $r1                            ; 6
                changed to: noop                                    ; 6
                Nothing
            add $$arg0 $r0 $r1                      ; 7
                Some(Const(10)) Some(Const(10))
                changed to: movi $$arg0 i20                         ; 7
                Nothing
            add $$arg1 $zero $r0                    ; 8
                Some(Const(0)) Some(Const(10))
                changed to: movi $$arg1 i10                         ; 8
                Nothing
            add $$arg2 $r0 $zero                    ; 9
                Some(Const(10)) Some(Const(0))
                changed to: movi $$arg2 i10                         ; 9
                Nothing
            add $$arg3 $r5 $r0                      ; 10
                None Some(Const(10))
                changed to: addi $$arg3 $r5 i10                     ; 10
                Defs
            add $$arg4 $r0 $r5                      ; 11
                Some(Const(10)) None
                changed to: addi $$arg4 $r5 i10                     ; 11
                Defs
            sub $$arg0 $r0 $r1                      ; 12
                Some(Const(10)) Some(Const(10))
                changed to: movi $$arg0 i0                          ; 12
                Nothing
            sub $$arg1 $r0 $zero                    ; 13
                Some(Const(10)) Some(Const(0))
                changed to: movi $$arg1 i10                         ; 13
                Nothing
            mul $$arg0 $r0 $r1                      ; 14
                Some(Const(10)) Some(Const(10))
                changed to: movi $$arg0 i100                        ; 14
                Nothing
            mul $$arg1 $one $r0                     ; 15
                Some(Const(1)) Some(Const(10))
                changed to: movi $$arg1 i10                         ; 15
                Nothing
            mul $$arg2 $r0 $one                     ; 16
                Some(Const(10)) Some(Const(1))
                changed to: movi $$arg2 i10                         ; 16
                Nothing
            mul $$arg3 $zero $r0                    ; 17
                Some(Const(0)) Some(Const(10))
                changed to: movi $$arg3 i0                          ; 17
                Nothing
            mul $$arg4 $r0 $zero                    ; 18
                Some(Const(10)) Some(Const(0))
                changed to: movi $$arg4 i0                          ; 18
                Nothing
            mul $$arg5 $r5 $r0                      ; 19
                None Some(Const(10))
                changed to: muli $$arg5 $r5 i10                     ; 19
                Defs
            mul $$arg0 $r0 $r5                      ; 20
                Some(Const(10)) None
                changed to: muli $$arg0 $r5 i10                     ; 20
                Defs
            div $$arg0 $r0 $r1                      ; 21
                Some(Const(10)) Some(Const(10))
                changed to: movi $$arg0 i1                          ; 21
                Nothing
            div $$arg1 $r0 $one                     ; 22
                Some(Const(10)) Some(Const(1))
                changed to: movi $$arg1 i10                         ; 22
                Nothing
            div $$arg2 $zero $r0                    ; 23
                Some(Const(0)) Some(Const(10))
                changed to: movi $$arg2 i0                          ; 23
                Nothing
            div $$arg2 $zero $zero                  ; 24
                Some(Const(0)) Some(Const(0))
                Full
            exp $$arg0 $r0 $r2                      ; 25
                None None
                Full
            exp $$arg0 $zero $zero                  ; 26
                Some(Const(0)) Some(Const(0))
                changed to: movi $$arg0 i1                          ; 26
                Nothing
            exp $$arg1 $r0 $zero                    ; 27
                None Some(Const(0))
                changed to: movi $$arg1 i1                          ; 27
                Nothing
            exp $$arg2 $r0 $one                     ; 28
                None Some(Const(1))
                changed to: move $$arg2 $r0                         ; 28
                Nothing
            exp $$arg3 $zero $r0                    ; 29
                Some(Const(0)) None
                changed to: movi $$arg3 i0                          ; 29
                Nothing
            exp $$arg4 $one $r0                     ; 30
                Some(Const(1)) None
                changed to: movi $$arg4 i1                          ; 30
                Nothing
            mlog $$arg0 $r3 $r2                     ; 31
                None None
                Full
            mod $$arg0 $r4 $r2                      ; 32
                None None
                Full
            mod $$arg1 $r4 $one                     ; 33
                None Some(Const(1))
                changed to: movi $$arg1 i0                          ; 33
                Nothing
            mroo $$arg0 $r4 $r2                     ; 34
                None None
                Full
            mroo $$arg1 $r4 $one                    ; 35
                None Some(Const(1))
                changed to: move $$arg1 $r4                         ; 35
                Nothing
            and $$arg0 $r0 $r2                      ; 36
                None None
                Full
            and $$arg1 $zero $r2                    ; 37
                Some(Const(0)) None
                changed to: movi $$arg1 i0                          ; 37
                Nothing
            and $$arg2 $r2 $zero                    ; 38
                None Some(Const(0))
                changed to: movi $$arg2 i0                          ; 38
                Nothing
            and $$arg3 $r5 $r0                      ; 39
                None None
                Full
            and $$arg4 $r0 $r5                      ; 40
                None None
                Full
            or $$arg0 $r0 $r2                       ; 41
                None None
                Full
            or $$arg1 $zero $r2                     ; 42
                Some(Const(0)) None
                changed to: move $$arg1 $r2                         ; 42
                Nothing
            or $$arg2 $r2 $zero                     ; 43
                None Some(Const(0))
                changed to: move $$arg2 $r2                         ; 43
                Nothing
            or $$arg3 $r5 $r0                       ; 44
                None None
                Full
            or $$arg4 $r0 $r5                       ; 45
                None None
                Full
            xor $$arg0 $r0 $r2                      ; 46
                None None
                Full
            xor $$arg1 $zero $r2                    ; 47
                Some(Const(0)) None
                changed to: move $$arg1 $r2                         ; 47
                Nothing
            xor $$arg2 $r2 $zero                    ; 48
                None Some(Const(0))
                changed to: move $$arg2 $r2                         ; 48
                Nothing
            xor $$arg3 $r5 $r0                      ; 49
                None None
                Full
            xor $$arg4 $r0 $r5                      ; 50
                None None
                Full
            sll $$arg0 $r0 $r2                      ; 51
                None None
                Full
            sll $$arg2 $r2 $zero                    ; 52
                None Some(Const(0))
                changed to: move $$arg2 $r2                         ; 52
                Nothing
            srl $$arg0 $r0 $r2                      ; 53
                None None
                Full
            srl $$arg2 $r2 $zero                    ; 54
                None Some(Const(0))
                changed to: move $$arg2 $r2                         ; 54
                Nothing
            eq $$arg0 $r0 $r2                       ; 55
                None None
                Full
            gt $$arg0 $r0 $r2                       ; 56
                None None
                Full
            lt $$arg0 $r0 $r2                       ; 57
                None None
                Full
        "#]]
        .assert_eq(&str);
    }
}
