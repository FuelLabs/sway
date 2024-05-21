//! An analysis to compute symbols that escape out from a function.
//! This could be into another function, or via `ptr_to_int` etc.
//! Any transformations involving such symbols are unsafe.

use indexmap::IndexSet;
use rustc_hash::FxHashSet;
use sway_types::{FxIndexMap, FxIndexSet};

use crate::{
    AnalysisResult, AnalysisResultT, AnalysisResults, BlockArgument, Context, FuelVmInstruction,
    Function, InstOp, Instruction, IrError, LocalVar, Pass, PassMutability, ScopedPass, Type,
    Value, ValueDatum,
};

pub const ESCAPED_SYMBOLS_NAME: &str = "escaped-symbols";

pub fn create_escaped_symbols_pass() -> Pass {
    Pass {
        name: ESCAPED_SYMBOLS_NAME,
        descr: "Symbols that escape or cannot be analyzed",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Analysis(compute_escaped_symbols_pass)),
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Symbol {
    Local(LocalVar),
    Arg(BlockArgument),
}

impl Symbol {
    pub fn get_type(&self, context: &Context) -> Type {
        match self {
            Symbol::Local(l) => l.get_type(context),
            Symbol::Arg(ba) => ba.ty,
        }
    }

    pub fn _get_name(&self, context: &Context, function: Function) -> String {
        match self {
            Symbol::Local(l) => function.lookup_local_name(context, l).unwrap().clone(),
            Symbol::Arg(ba) => format!("{}[{}]", ba.block.get_label(context), ba.idx),
        }
    }
}

/// Get [Symbol]s, both [Symbol::Local]s and [Symbol::Arg]s, reachable
/// from the `val` via chain of [InstOp::GetElemPtr] (GEP) instructions.
/// A `val` can, via GEP instructions, refer indirectly to none, or one
/// or more symbols.
///
/// Note that this function does not return [Symbol]s potentially reachable
/// via referencing (`&`), dereferencing (`*`), and raw pointers (`__addr_of`)
/// and is thus suitable for all IR analysis and manipulation that deals
/// strictly with GEP access.
///
/// To acquire all [Symbol]s reachable from the `val`, use [get_referred_symbols] instead.
pub fn get_gep_referred_symbols(context: &Context, val: Value) -> FxIndexSet<Symbol> {
    match get_symbols(context, val, true) {
        ReferredSymbols::Complete(symbols) => symbols,
        _ => unreachable!(
            "In the case of GEP access, the set of returned symbols is always complete."
        ),
    }
}

/// Provides [Symbol]s, both [Symbol::Local]s and [Symbol::Arg]s, reachable
/// from a certain [Value] via chain of [InstOp::GetElemPtr] (GEP) instructions
/// or via [InstOp::IntToPtr] and [InstOp::PtrToInt] instruction patterns
/// specific to references, both referencing (`&`) and dereferencing (`*`),
/// and raw pointers, via `__addr_of`.
pub enum ReferredSymbols {
    /// Guarantees that all [Symbol]s reachable from the particular [Value]
    /// are collected, thus, that there are no escapes or pointer accesses
    /// in the scope that _might_ result in symbols indirectly related to
    /// the [Value] but not reachable only via GEP, or references, or
    /// raw pointers only.
    Complete(FxIndexSet<Symbol>),
    /// Denotes that there _might_ be [Symbol]s out of returned ones that
    /// are related to the particular [Value], but not reachable only via GEP,
    /// or references, or raw pointers.
    Incomplete(FxIndexSet<Symbol>),
}

impl ReferredSymbols {
    // TODO: Check all the usages of this method and replace it with the
    //       checked access to either complete or incomplete symbols.
    //       This is a temporary convenience method until
    //       we decide case by case how to deal with incomplete set of symbols.
    //       See: https://github.com/FuelLabs/sway/issues/5924
    pub fn any(self) -> FxIndexSet<Symbol> {
        match self {
            ReferredSymbols::Complete(symbols) | ReferredSymbols::Incomplete(symbols) => symbols,
        }
    }
}

/// Get [Symbol]s, both [Symbol::Local]s and [Symbol::Arg]s, reachable
/// from the `val` via chain of [InstOp::GetElemPtr] (GEP) instructions
/// or via [InstOp::IntToPtr] and [InstOp::PtrToInt] instruction patterns
/// specific to references, both referencing (`&`) and dereferencing (`*`),
/// and raw pointers, via `__addr_of`.
/// A `val` can, via these instructions, refer indirectly to none, or one
/// or more symbols.
///
/// Note that *this function does not perform any escape analysis*. E.g., if a
/// local symbol gets passed by `raw_ptr` or `&T` to a function and returned
/// back from the function via the same `raw_ptr` or `&T` the value returned
/// from the function will not be tracked back to the original symbol and the
/// symbol will not be collected as referred.
///
/// This means that, even if the result contains [Symbol]s, it _might_ be that
/// there are still other [Symbol]s in scope related to the `val`. E.g., in case
/// of branching, where the first branch directly returns `& local_var_a`
/// and the second branch, indirectly over a function call as explained above,
/// `& local_var_b`, only the `local_var_a` will be returned as a result.
///
/// Therefore, the function returns the [ReferredSymbols] enum to denote
/// if the returned set of symbols is guaranteed to be complete, or if it is
/// incomplete.
pub fn get_referred_symbols(context: &Context, val: Value) -> ReferredSymbols {
    get_symbols(context, val, false)
}

/// Get [Symbol]s, both [Symbol::Local]s and [Symbol::Arg]s, reachable
/// from the `val`.
///
/// If `gep_only` is `true` only the [Symbol]s reachable via GEP instructions
/// are returned. Otherwise, the result also contains [Symbol]s reachable
/// via referencing (`&`) and dereferencing (`*`).
fn get_symbols(context: &Context, val: Value, gep_only: bool) -> ReferredSymbols {
    fn get_symbols_rec(
        context: &Context,
        symbols: &mut FxIndexSet<Symbol>,
        visited: &mut FxHashSet<Value>,
        val: Value,
        gep_only: bool,
        is_complete: &mut bool,
    ) {
        fn get_argument_symbols(
            context: &Context,
            symbols: &mut FxIndexSet<Symbol>,
            visited: &mut FxHashSet<Value>,
            arg: BlockArgument,
            gep_only: bool,
            is_complete: &mut bool,
        ) {
            if arg.block.get_label(context) == "entry" {
                symbols.insert(Symbol::Arg(arg));
            } else {
                arg.block
                    .pred_iter(context)
                    .map(|pred| arg.get_val_coming_from(context, pred).unwrap())
                    .for_each(|v| {
                        get_symbols_rec(context, symbols, visited, v, gep_only, is_complete)
                    })
            }
        }

        if visited.contains(&val) {
            return;
        }
        visited.insert(val);
        match context.values[val.0].value {
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetLocal(local),
                ..
            }) => {
                symbols.insert(Symbol::Local(local));
            }
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetElemPtr { base, .. },
                ..
            }) => get_symbols_rec(context, symbols, visited, base, gep_only, is_complete),
            // The below chain of instructions are specific to
            // referencing, dereferencing, and `__addr_of` and do not occur
            // in other kinds of IR generation.  E.g., `IntToPtr` could be emitted when
            // GTF intrinsic is compiled, but do not produce
            // the below patterns which are specific to references and raw pointers.
            ValueDatum::Instruction(Instruction {
                op: InstOp::IntToPtr(int_value, _),
                ..
            }) if !gep_only => {
                // Ignore this path if only GEP chain is requested.
                match context.values[int_value.0].value {
                    ValueDatum::Instruction(Instruction {
                        op: InstOp::Load(loaded_from),
                        ..
                    }) => get_symbols_rec(
                        context,
                        symbols,
                        visited,
                        loaded_from,
                        gep_only,
                        is_complete,
                    ),
                    ValueDatum::Instruction(Instruction {
                        op: InstOp::PtrToInt(ptr_value, _),
                        ..
                    }) => {
                        get_symbols_rec(context, symbols, visited, ptr_value, gep_only, is_complete)
                    }
                    ValueDatum::Argument(arg) => {
                        get_argument_symbols(context, symbols, visited, arg, gep_only, is_complete)
                    }
                    // In other cases, e.g., getting the integer address from an unsafe pointer
                    // arithmetic, or as a function result, etc. we bail out and mark the
                    // collection as not being guaranteed to be a complete set of all referred symbols.
                    _ => {
                        *is_complete = false;
                    }
                }
            }
            // In case of converting pointer to int for references and raw pointers,
            // we consider the pointed symbols to be reachable from the `ptr_value`.
            ValueDatum::Instruction(Instruction {
                op: InstOp::PtrToInt(ptr_value, _),
                ..
            }) if !gep_only => {
                get_symbols_rec(context, symbols, visited, ptr_value, gep_only, is_complete)
            }
            ValueDatum::Argument(arg) => {
                get_argument_symbols(context, symbols, visited, arg, gep_only, is_complete)
            }
            _ if !gep_only => {
                // Same as above, we cannot track the value up the chain and cannot guarantee
                // that the value is not coming from some of the symbols.
                *is_complete = false;
            }
            // In the case of GEP only access, the returned set is always complete.
            _ => (),
        }
    }

    let mut visited = FxHashSet::default();
    let mut symbols = IndexSet::default();
    let mut is_complete = true;

    get_symbols_rec(
        context,
        &mut symbols,
        &mut visited,
        val,
        gep_only,
        &mut is_complete,
    );

    if is_complete {
        ReferredSymbols::Complete(symbols)
    } else {
        ReferredSymbols::Incomplete(symbols)
    }
}

pub fn get_gep_symbol(context: &Context, val: Value) -> Option<Symbol> {
    let syms = get_gep_referred_symbols(context, val);
    (syms.len() == 1)
        .then(|| syms.iter().next().cloned())
        .flatten()
}

pub fn get_referred_symbol(context: &Context, val: Value) -> Option<Symbol> {
    let syms = get_referred_symbols(context, val).any();
    (syms.len() == 1)
        .then(|| syms.iter().next().cloned())
        .flatten()
}

pub type EscapedSymbols = FxHashSet<Symbol>;
impl AnalysisResultT for EscapedSymbols {}

pub fn compute_escaped_symbols_pass(
    context: &Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<AnalysisResult, IrError> {
    Ok(Box::new(compute_escaped_symbols(context, &function)))
}

pub fn compute_escaped_symbols(context: &Context, function: &Function) -> EscapedSymbols {
    let mut result = FxHashSet::default();

    let add_from_val = |result: &mut FxHashSet<Symbol>, val: &Value| {
        get_referred_symbols(context, *val)
            .any()
            .iter()
            .for_each(|s| {
                result.insert(*s);
            });
    };

    for (_block, inst) in function.instruction_iter(context) {
        match &inst.get_instruction(context).unwrap().op {
            InstOp::AsmBlock(_, args) => {
                for arg_init in args.iter().filter_map(|arg| arg.initializer) {
                    add_from_val(&mut result, &arg_init)
                }
            }
            InstOp::UnaryOp { .. } => (),
            InstOp::BinaryOp { .. } => (),
            InstOp::BitCast(_, _) => (),
            InstOp::Branch(_) => (),
            InstOp::Call(_, args) => args.iter().for_each(|v| add_from_val(&mut result, v)),
            InstOp::CastPtr(_, _) => (),
            InstOp::Cmp(_, _, _) => (),
            InstOp::ConditionalBranch { .. } => (),
            InstOp::ContractCall { params, .. } => add_from_val(&mut result, params),
            InstOp::FuelVm(_) => (),
            InstOp::GetLocal(_) => (),
            InstOp::GetElemPtr { .. } => (),
            InstOp::IntToPtr(_, _) => (),
            InstOp::Load(_) => (),
            InstOp::MemCopyBytes { .. } => (),
            InstOp::MemCopyVal { .. } => (),
            InstOp::Nop => (),
            InstOp::PtrToInt(v, _) => add_from_val(&mut result, v),
            InstOp::Ret(_, _) => (),
            InstOp::Store { .. } => (),
        }
    }

    result
}

/// Pointers that may possibly be loaded from.
pub fn get_loaded_ptr_values(context: &Context, val: Value) -> Vec<Value> {
    match &val.get_instruction(context).unwrap().op {
        InstOp::UnaryOp { .. }
        | InstOp::BinaryOp { .. }
        | InstOp::BitCast(_, _)
        | InstOp::Branch(_)
        | InstOp::ConditionalBranch { .. }
        | InstOp::Cmp(_, _, _)
        | InstOp::Nop
        | InstOp::CastPtr(_, _)
        | InstOp::GetLocal(_)
        | InstOp::GetElemPtr { .. }
        | InstOp::IntToPtr(_, _) => vec![],
        InstOp::PtrToInt(src_val_ptr, _) => vec![*src_val_ptr],
        InstOp::ContractCall {
            params,
            coins,
            asset_id,
            ..
        } => vec![*params, *coins, *asset_id],
        InstOp::Call(_, args) => args.clone(),
        InstOp::AsmBlock(_, args) => args.iter().filter_map(|val| val.initializer).collect(),
        InstOp::MemCopyBytes { src_val_ptr, .. }
        | InstOp::MemCopyVal { src_val_ptr, .. }
        | InstOp::Ret(src_val_ptr, _)
        | InstOp::Load(src_val_ptr)
        | InstOp::FuelVm(FuelVmInstruction::Log {
            log_val: src_val_ptr,
            ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateLoadWord(src_val_ptr))
        | InstOp::FuelVm(FuelVmInstruction::StateStoreWord {
            key: src_val_ptr, ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateLoadQuadWord {
            key: src_val_ptr, ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateClear {
            key: src_val_ptr, ..
        }) => vec![*src_val_ptr],
        InstOp::FuelVm(FuelVmInstruction::StateStoreQuadWord {
            stored_val: memopd1,
            key: memopd2,
            ..
        })
        | InstOp::FuelVm(FuelVmInstruction::Smo {
            recipient: memopd1,
            message: memopd2,
            ..
        }) => vec![*memopd1, *memopd2],
        InstOp::Store { dst_val_ptr: _, .. } => vec![],
        InstOp::FuelVm(FuelVmInstruction::Gtf { .. })
        | InstOp::FuelVm(FuelVmInstruction::ReadRegister(_))
        | InstOp::FuelVm(FuelVmInstruction::Revert(_) | FuelVmInstruction::JmpMem) => vec![],
        InstOp::FuelVm(FuelVmInstruction::WideUnaryOp { arg, .. }) => vec![*arg],
        InstOp::FuelVm(FuelVmInstruction::WideBinaryOp { arg1, arg2, .. })
        | InstOp::FuelVm(FuelVmInstruction::WideCmpOp { arg1, arg2, .. }) => {
            vec![*arg1, *arg2]
        }
        InstOp::FuelVm(FuelVmInstruction::WideModularOp {
            arg1, arg2, arg3, ..
        }) => vec![*arg1, *arg2, *arg3],
        InstOp::FuelVm(FuelVmInstruction::Retd { ptr, .. }) => vec![*ptr],
    }
}

/// Symbols that may possibly be loaded from.
pub fn get_loaded_symbols(context: &Context, val: Value) -> FxIndexSet<Symbol> {
    let mut res = IndexSet::default();
    for val in get_loaded_ptr_values(context, val) {
        for sym in get_referred_symbols(context, val).any() {
            res.insert(sym);
        }
    }
    res
}

/// Pointers that may possibly be stored to.
pub fn get_stored_ptr_values(context: &Context, val: Value) -> Vec<Value> {
    match &val.get_instruction(context).unwrap().op {
        InstOp::UnaryOp { .. }
        | InstOp::BinaryOp { .. }
        | InstOp::BitCast(_, _)
        | InstOp::Branch(_)
        | InstOp::ConditionalBranch { .. }
        | InstOp::Cmp(_, _, _)
        | InstOp::Nop
        | InstOp::PtrToInt(_, _)
        | InstOp::Ret(_, _)
        | InstOp::CastPtr(_, _)
        | InstOp::GetLocal(_)
        | InstOp::GetElemPtr { .. }
        | InstOp::IntToPtr(_, _) => vec![],
        InstOp::ContractCall { params, .. } => vec![*params],
        InstOp::Call(_, args) => args.clone(),
        InstOp::AsmBlock(_, args) => args.iter().filter_map(|val| val.initializer).collect(),
        InstOp::MemCopyBytes { dst_val_ptr, .. }
        | InstOp::MemCopyVal { dst_val_ptr, .. }
        | InstOp::Store { dst_val_ptr, .. } => vec![*dst_val_ptr],
        InstOp::Load(_) => vec![],
        InstOp::FuelVm(vmop) => match vmop {
            FuelVmInstruction::Gtf { .. }
            | FuelVmInstruction::Log { .. }
            | FuelVmInstruction::ReadRegister(_)
            | FuelVmInstruction::Revert(_)
            | FuelVmInstruction::JmpMem
            | FuelVmInstruction::Smo { .. }
            | FuelVmInstruction::StateClear { .. } => vec![],
            FuelVmInstruction::StateLoadQuadWord { load_val, .. } => vec![*load_val],
            FuelVmInstruction::StateLoadWord(_) | FuelVmInstruction::StateStoreWord { .. } => {
                vec![]
            }
            FuelVmInstruction::StateStoreQuadWord { stored_val: _, .. } => vec![],
            FuelVmInstruction::WideUnaryOp { result, .. }
            | FuelVmInstruction::WideBinaryOp { result, .. }
            | FuelVmInstruction::WideModularOp { result, .. } => vec![*result],
            FuelVmInstruction::WideCmpOp { .. } => vec![],
            _ => vec![],
        },
    }
}

/// Symbols that may possibly be stored to.
pub fn get_stored_symbols(context: &Context, val: Value) -> FxIndexSet<Symbol> {
    let mut res = IndexSet::default();
    for val in get_stored_ptr_values(context, val) {
        for sym in get_referred_symbols(context, val).any() {
            res.insert(sym);
        }
    }
    res
}

/// Combine a series of GEPs into one.
pub fn combine_indices(context: &Context, val: Value) -> Option<Vec<Value>> {
    match &context.values[val.0].value {
        ValueDatum::Instruction(Instruction {
            op: InstOp::GetLocal(_),
            ..
        }) => Some(vec![]),
        ValueDatum::Instruction(Instruction {
            op:
                InstOp::GetElemPtr {
                    base,
                    elem_ptr_ty: _,
                    indices,
                },
            ..
        }) => {
            let mut base_indices = combine_indices(context, *base)?;
            base_indices.append(&mut indices.clone());
            Some(base_indices)
        }
        ValueDatum::Argument(_) => Some(vec![]),
        _ => None,
    }
}

/// Given a memory pointer instruction, compute the offset of indexed element,
/// for each symbol that it may alias to.
/// If for any symbol we can't compute it, return None.
pub fn get_memory_offsets(context: &Context, val: Value) -> Option<FxIndexMap<Symbol, u64>> {
    let syms = get_gep_referred_symbols(context, val);

    let mut res: FxIndexMap<Symbol, u64> = FxIndexMap::default();
    for sym in syms {
        let offset = sym
            .get_type(context)
            .get_pointee_type(context)?
            .get_value_indexed_offset(context, &combine_indices(context, val)?)?;
        res.insert(sym, offset);
    }
    Some(res)
}

/// Can memory ranges [val1, val1+len1] and [val2, val2+len2] overlap?
/// Conservatively returns true if cannot statically determine.
pub fn may_alias(context: &Context, val1: Value, len1: u64, val2: Value, len2: u64) -> bool {
    let (Some(mem_offsets_1), Some(mem_offsets_2)) = (
        get_memory_offsets(context, val1),
        get_memory_offsets(context, val2),
    ) else {
        return true;
    };

    for (sym1, off1) in mem_offsets_1 {
        if let Some(off2) = mem_offsets_2.get(&sym1) {
            // does off1 + len1 overlap with off2 + len2?
            if (off1 <= *off2 && (off1 + len1 > *off2)) || (*off2 <= off1 && (*off2 + len2 > off1))
            {
                return true;
            }
        }
    }
    false
}

/// Are memory ranges [val1, val1+len1] and [val2, val2+len2] exactly the same?
/// Conservatively returns false if cannot statically determine.
pub fn must_alias(context: &Context, val1: Value, len1: u64, val2: Value, len2: u64) -> bool {
    let (Some(mem_offsets_1), Some(mem_offsets_2)) = (
        get_memory_offsets(context, val1),
        get_memory_offsets(context, val2),
    ) else {
        return false;
    };

    if mem_offsets_1.len() != 1 || mem_offsets_2.len() != 1 {
        return false;
    }

    let (sym1, off1) = mem_offsets_1.iter().next().unwrap();
    let (sym2, off2) = mem_offsets_2.iter().next().unwrap();

    // does off1 + len1 overlap with off2 + len2?
    sym1 == sym2 && off1 == off2 && len1 == len2
}

/// For a pointer argument `ptr_val`, what's the size of its pointee.
pub fn pointee_size(context: &Context, ptr_val: Value) -> u64 {
    ptr_val
        .get_type(context)
        .unwrap()
        .get_pointee_type(context)
        .expect("Expected arg to be a pointer")
        .size(context)
        .in_bytes()
}
