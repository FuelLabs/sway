use super::super::abstract_instruction_set::AbstractInstructionSet;
use crate::asm_lang::{
    ConstantRegister, ControlFlowOp, JumpType, Label, Op, VirtualImmediate18,
    VirtualOp, VirtualRegister,
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
                known_values.values.clear();
            }
            ResetKnown::NonVirtual => {
                Self::Defs.apply(op, known_values);
                known_values
                    .values
                    .retain(|k, _| matches!(k, VirtualRegister::Virtual(_)));
            }
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
        }
    }
}

impl AbstractInstructionSet {
    /// Symbolically interpret code and propagate known register values.
    pub(crate) fn constant_propagate(mut self) -> AbstractInstructionSet {
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
                        return Some(true);
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
                        return Some(true);
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
                        return Some(true);
                    }
                    transform_operator!{gen; $op, $opI; $dst, $l, $r, $lv, $rv; $($rest)*}
                };
                // Transform from $op to $opI because $op is commutative
                (gen; $op:ident, $opI:ident; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident; commutative: true; $($rest:tt)*) => {
                    if let (Some(KnownRegValue::Const($lv)), _) = (&$lv, &$rv) {
                        op.opcode = Either::Left(VirtualOp::$opI($dst.clone(), $r.clone(), (*$lv).try_into().ok()?));
                        return Some(false);
                    }
                    transform_operator!{gen; $op, $opI; $dst, $l, $r, $lv, $rv; $($rest)*}
                };
                (gen; $op:ident, $opI:ident; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident;) => {
                };
                (gen; $op:ident, $opI:ident; $($rest:tt)*) => {
                    compile_error!(stringify!($($rest)*))
                };
                (replace_imm; None; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident) => {
                };
                (replace_imm; $opI:ident; $dst:ident, $l:ident, $r:ident, $lv: ident, $rv:ident) => {
                    if let (_, Some(KnownRegValue::Const(rv))) = (&$lv, &$rv) {
                        op.opcode = Either::Left(VirtualOp::$opI($dst.clone(), $l.clone(), (*rv).try_into().ok()?));
                        return Some(false);
                    }
                };
                ($op:ident, $opI:ident; $($rest:tt)*) => {{
                    let mut f = || -> Option<bool> {
                        match &op.opcode {
                            Either::Left(VirtualOp::$op(dst, l, r)) => {
                                let lv = known_values.resolve(&l);
                                let rv = known_values.resolve(&r);

                                transform_operator!{gen; $op, $opI; dst, l, r, lv, rv; $($rest)*}
                                transform_operator!{replace_imm; $opI; dst, l, r, lv, rv}
                            }
                            _ => {}
                        };
                        Some(false)
                    };
                    f().unwrap_or_default()
                }};
            }

            // Propagate constant of some ops
            // Also transform them if they registers are known
            let skip_reset = match op.opcode.clone() {
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

                    true
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

                    true
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
                },
                Either::Left(VirtualOp::MLOG(..)) => transform_operator! {MLOG, None;
                    both_known: u64::checked_ilog;
                },
                Either::Left(VirtualOp::MOD(..)) => transform_operator! {MOD, MODI;
                    both_known: u64::checked_rem;
                },
                Either::Left(VirtualOp::MROO(..)) => transform_operator! {MROO, None;
                    both_known: checked_nth_root;
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
                _ => false,
            };

            if !skip_reset {
                let reset = match &op.opcode {
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
    use prettydiff::basic::DiffOp;

    fn optimise(
        ops: impl IntoIterator<Item = Op>,
        f: impl FnOnce(AbstractInstructionSet) -> AbstractInstructionSet,
    ) -> String {
        let ops = AbstractInstructionSet {
            function: None,
            ops: ops.into_iter().collect(),
        };

        let old = format!("{ops}");
        let ops = f(ops);
        let new = format!("{ops}");

        let lines = prettydiff::diff_lines(&old, &new);

        let mut s = String::new();
        for d in lines.diff() {
            match d {
                DiffOp::Insert(items) => {
                    for item in items {
                        s.push_str(format!("+ {}\n", item).as_str());
                    }
                }
                DiffOp::Replace(items, items1) => {
                    for item in items {
                        s.push_str(format!("- {}\n", item).as_str());
                    }

                    for item in items1 {
                        s.push_str(format!("+ {}\n", item).as_str());
                    }
                }
                DiffOp::Remove(items) => {
                    for item in items {
                        s.push_str(format!("- {}\n", item).as_str());
                    }
                }
                DiffOp::Equal(items) => {
                    for item in items {
                        s.push_str(format!("{}\n", item).as_str());
                    }
                }
            }
        }

        s
    }

    #[test]
    fn constant_propagate_transform_movi_to_noop() {
        let actual = optimise(
            [
                VirtualOp::movi("0", 10).into(),
                VirtualOp::movi("0", 10).into(),
            ],
            |ops| ops.constant_propagate(),
        );

        expect![
            ".program:
movi $r0 i10
- movi $r0 i10
"
        ]
        .assert_eq(&actual);
    }

    #[test]
    fn constant_propagate_transform_move_to_noop() {
        let actual = optimise(
            [
                VirtualOp::movi("0", 10).into(),
                VirtualOp::movi("1", 10).into(),
                VirtualOp::r#move("1", "0").into(),
            ],
            |ops| ops.constant_propagate(),
        );

        expect![
            ".program:
movi $r0 i10
movi $r1 i10
- move $r1 $r0
"
        ]
        .assert_eq(&actual);
    }

    #[test]
    fn constant_propagate_transform_move_to_movi() {
        let actual = optimise(
            [
                VirtualOp::movi("0", 10).into(),
                VirtualOp::r#move(ConstantRegister::FuncArg0, "0").into(),
            ],
            |ops| ops.constant_propagate(),
        );

        expect![
            ".program:
movi $r0 i10
- move $$arg0 $r0
+ movi $$arg0 i10
"
        ]
        .assert_eq(&actual);
    }

    #[test]
    fn constant_propagate_transform_add_to_addi_on_op_0() {
        let actual = optimise(
            [
                VirtualOp::movi("0", 10).into(),
                VirtualOp::add(ConstantRegister::FuncArg0, "0", "1").into(),
            ],
            |ops| ops.constant_propagate(),
        );

        expect![
            ".program:
movi $r0 i10
- add $$arg0 $r0 $r1
+ addi $$arg0 $r1 i10
"
        ]
        .assert_eq(&actual);
    }

    #[test]
    fn constant_propagate_transform_add_to_addi_on_op_1() {
        let actual = optimise(
            [
                VirtualOp::movi("0", 10).into(),
                VirtualOp::add(ConstantRegister::FuncArg0, "1", "0").into(),
            ],
            |ops| ops.constant_propagate(),
        );

        expect![
            ".program:
movi $r0 i10
- add $$arg0 $r1 $r0
+ addi $$arg0 $r1 i10
"
        ]
        .assert_eq(&actual);
    }

    #[test]
    fn constant_propagate_transform_add_to_movi_on_both_ops() {
        let actual = optimise(
            [
                VirtualOp::movi("0", 10).into(),
                VirtualOp::add(ConstantRegister::FuncArg0, "0", "0").into(),
            ],
            |ops| ops.constant_propagate(),
        );

        expect![
            ".program:
movi $r0 i10
- add $$arg0 $r0 $r0
+ movi $$arg0 i20
"
        ]
        .assert_eq(&actual);
    }

    #[test]
    fn constant_propagate_transform_sub_to_movi_on_op_1() {
        let actual = optimise(
            [
                VirtualOp::movi("0", 10).into(),
                VirtualOp::sub(ConstantRegister::FuncArg0, "0", "0").into(),
            ],
            |ops| ops.constant_propagate(),
        );

        expect![
            ".program:
movi $r0 i10
- sub $$arg0 $r0 $r0
+ movi $$arg0 i0
"
        ]
        .assert_eq(&actual);
    }

    #[test]
    fn constant_propagate_transform_mul_to_movi_on_op_1() {
        let actual = optimise(
            [
                VirtualOp::movi("0", 10).into(),
                VirtualOp::mul(ConstantRegister::FuncArg0, "0", "0").into(),
            ],
            |ops| ops.constant_propagate(),
        );

        expect![
            ".program:
movi $r0 i10
- mul $$arg0 $r0 $r0
+ movi $$arg0 i100
"
        ]
        .assert_eq(&actual);
    }

    #[test]
    fn constant_propagate_transform_div_to_movi_on_op_1() {
        let actual = optimise(
            [
                VirtualOp::movi("0", 10).into(),
                VirtualOp::div(ConstantRegister::FuncArg0, "0", "0").into(),
            ],
            |ops| ops.constant_propagate(),
        );

        expect![
            ".program:
movi $r0 i10
- div $$arg0 $r0 $r0
+ movi $$arg0 i1
"
        ]
        .assert_eq(&actual);
    }

    #[test]
    fn constant_propagate_transform_exp_to_movi_on_op_1() {
        let actual = optimise(
            [
                VirtualOp::movi("0", 3).into(),
                VirtualOp::exp(ConstantRegister::FuncArg0, "0", "0").into(),
            ],
            |ops| ops.constant_propagate(),
        );

        expect![
            ".program:
movi $r0 i3
- exp $$arg0 $r0 $r0
+ movi $$arg0 i27
"
        ]
        .assert_eq(&actual);
    }
}
