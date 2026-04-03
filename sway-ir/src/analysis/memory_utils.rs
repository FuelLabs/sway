//! An analysis to compute symbols that escape out from a function.
//! This could be into another function, or via `ptr_to_int` etc.
//! Any transformations involving such symbols are unsafe.

use indexmap::IndexSet;
use rustc_hash::FxHashSet;
use sway_types::{FxIndexMap, FxIndexSet};

use crate::{
    AnalysisResult, AnalysisResultT, AnalysisResults, BlockArgument, Context, FuelVmInstruction,
    Function, InitAggr, InstOp, Instruction, IrError, LocalVar, Pass, PassMutability, ScopedPass,
    Type, Value, ValueDatum,
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
/// If the `val` is not a pointer, an empty set is returned.
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
    pub fn new(is_complete: bool, symbols: FxIndexSet<Symbol>) -> Self {
        if is_complete {
            Self::Complete(symbols)
        } else {
            Self::Incomplete(symbols)
        }
    }

    /// Returns the referred [Symbol]s and the information if they are
    /// complete (true) or incomplete (false).
    pub fn consume(self) -> (bool, FxIndexSet<Symbol>) {
        let is_complete = matches!(self, ReferredSymbols::Complete(_));
        let syms = match self {
            ReferredSymbols::Complete(syms) | ReferredSymbols::Incomplete(syms) => syms,
        };

        (is_complete, syms)
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
///
/// If the `val` is not a pointer, an empty set is returned and marked as
/// [ReferredSymbols::Complete].
pub fn get_referred_symbols(context: &Context, val: Value) -> ReferredSymbols {
    get_symbols(context, val, false)
}

/// Get [Symbol]s, both [Symbol::Local]s and [Symbol::Arg]s, reachable
/// from the `val`.
///
/// If `gep_only` is `true` only the [Symbol]s reachable via GEP instructions
/// are returned. Otherwise, the result also contains [Symbol]s reachable
/// via referencing (`&`) and dereferencing (`*`).
///
/// If the `val` is not a pointer, an empty set is returned and marked as
/// [ReferredSymbols::Complete].
fn get_symbols(context: &Context, val: Value, gep_only: bool) -> ReferredSymbols {
    // The input to this recursive function is always a pointer.
    // The function tracks backwards where the pointer is coming from.
    fn get_symbols_rec(
        context: &Context,
        symbols: &mut FxIndexSet<Symbol>,
        visited: &mut FxHashSet<Value>,
        ptr: Value,
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

        fn get_symbols_from_u64_address_argument(
            context: &Context,
            symbols: &mut FxIndexSet<Symbol>,
            visited: &mut FxHashSet<Value>,
            u64_address_arg: BlockArgument,
            is_complete: &mut bool,
        ) {
            if u64_address_arg.block.get_label(context) == "entry" {
                // The u64 address is coming from a function argument.
                // Same as in the case of a pointer coming from a function argument,
                // we collect it.
                symbols.insert(Symbol::Arg(u64_address_arg));
            } else {
                u64_address_arg
                    .block
                    .pred_iter(context)
                    .map(|pred| u64_address_arg.get_val_coming_from(context, pred).unwrap())
                    .for_each(|v| {
                        get_symbols_from_u64_address_rec(context, symbols, visited, v, is_complete)
                    })
            }
        }

        // The input to this recursive function is always a `u64` holding an address.
        // The below chain of instructions are specific to patterns where pointers
        // are obtained from `u64` addresses and vice versa. This includes:
        //  - referencing and dereferencing
        //  - raw pointers (`__addr_of`)
        //  - GTF intrinsic
        fn get_symbols_from_u64_address_rec(
            context: &Context,
            symbols: &mut FxIndexSet<Symbol>,
            visited: &mut FxHashSet<Value>,
            u64_address: Value,
            is_complete: &mut bool,
        ) {
            match context.values[u64_address.0].value {
                // Follow the sources of the address, and for every source address,
                // recursively come back to this function.
                ValueDatum::Argument(arg) => get_symbols_from_u64_address_argument(
                    context,
                    symbols,
                    visited,
                    arg,
                    is_complete,
                ),
                // 1. Patterns related to references and raw pointers.
                ValueDatum::Instruction(Instruction {
                    // The address is coming from a `raw_pointer` or `&T` variable.
                    op: InstOp::Load(_loaded_from),
                    ..
                }) => {
                    // TODO: https://github.com/FuelLabs/sway/issues/6065
                    //       We want to track sources of loaded addresses.
                    //       Currently we don't and simply mark the result as incomplete.
                    *is_complete = false;
                }
                ValueDatum::Instruction(Instruction {
                    op: InstOp::PtrToInt(ptr_value, _),
                    ..
                }) => get_symbols_rec(context, symbols, visited, ptr_value, false, is_complete),
                // 2. The address is coming from a GTF instruction.
                ValueDatum::Instruction(Instruction {
                    // There cannot be a symbol behind it, and so the returned set is complete.
                    op: InstOp::FuelVm(FuelVmInstruction::Gtf { .. }),
                    ..
                }) => (),
                // In other cases, e.g., getting the integer address from an unsafe pointer
                // arithmetic, or as a function result, etc. we bail out and mark the
                // collection as not being guaranteed to be a complete set of all referred symbols.
                _ => {
                    *is_complete = false;
                }
            }
        }

        if visited.contains(&ptr) {
            return;
        }
        visited.insert(ptr);
        match context.values[ptr.0].value {
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
            ValueDatum::Instruction(Instruction {
                op: InstOp::IntToPtr(u64_address, _),
                ..
            }) if !gep_only => get_symbols_from_u64_address_rec(
                context,
                symbols,
                visited,
                u64_address,
                is_complete,
            ),
            // We've reached a configurable at the top of the chain.
            // There cannot be a symbol behind it, and so the returned set is complete.
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetConfig(_, _),
                ..
            }) if !gep_only => (),
            // We've reached a global at the top of the chain.
            // There cannot be a symbol behind it, and so the returned set is complete.
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetGlobal(_),
                ..
            }) if !gep_only => (),
            // We've reached a storage key at the top of the chain.
            // There cannot be a symbol behind it, and so the returned set is complete.
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetStorageKey(_),
                ..
            }) if !gep_only => (),
            // Note that in this case, the pointer itself is coming from a `Load`,
            // and not an address. So, we just continue following the pointer.
            ValueDatum::Instruction(Instruction {
                op: InstOp::Load(loaded_from),
                ..
            }) if !gep_only => get_symbols_rec(
                context,
                symbols,
                visited,
                loaded_from,
                gep_only,
                is_complete,
            ),
            ValueDatum::Instruction(Instruction {
                op: InstOp::CastPtr(ptr_to_cast, _),
                ..
            }) if !gep_only => get_symbols_rec(
                context,
                symbols,
                visited,
                ptr_to_cast,
                gep_only,
                is_complete,
            ),
            ValueDatum::Argument(arg) => {
                get_argument_symbols(context, symbols, visited, arg, gep_only, is_complete)
            }
            // We've reached a constant at the top of the chain.
            // There cannot be a symbol behind it, and so the returned set is complete.
            ValueDatum::Constant(_) if !gep_only => (),
            _ if !gep_only => {
                // In other cases, e.g., getting the pointer from an ASM block,
                // or as a function result, etc., we cannot track the value up the chain
                // and cannot guarantee that the value is not coming from some of the symbols.
                // So, we bail out and mark the collection as not being guaranteed to be
                // a complete set of all referred symbols.
                *is_complete = false;
            }
            // In the case of GEP only access, the returned set is always complete.
            _ => (),
        }
    }

    if !val.get_type(context).is_some_and(|t| t.is_ptr(context)) {
        return ReferredSymbols::new(true, IndexSet::default());
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

    ReferredSymbols::new(is_complete, symbols)
}

pub fn get_gep_symbol(context: &Context, val: Value) -> Option<Symbol> {
    let syms = get_gep_referred_symbols(context, val);
    (syms.len() == 1)
        .then(|| syms.iter().next().cloned())
        .flatten()
}

/// Return [Symbol] referred by `val` if there is _exactly one_ symbol referred,
/// or `None` if there are no [Symbol]s referred or if there is more than one
/// referred.
pub fn get_referred_symbol(context: &Context, val: Value) -> Option<Symbol> {
    let syms = get_referred_symbols(context, val);
    match syms {
        ReferredSymbols::Complete(syms) => (syms.len() == 1)
            .then(|| syms.iter().next().cloned())
            .flatten(),
        // It might be that we have more than one referred symbol here.
        ReferredSymbols::Incomplete(_) => None,
    }
}

pub enum EscapedSymbols {
    /// Guarantees that all escaping [Symbol]s are collected.
    Complete(FxHashSet<Symbol>),
    /// Denotes that there _might_ be additional escaping [Symbol]s
    /// out of the collected ones.
    Incomplete(FxHashSet<Symbol>),
}

impl AnalysisResultT for EscapedSymbols {}

pub fn compute_escaped_symbols_pass(
    context: &Context,
    _analyses: &AnalysisResults,
    function: Function,
) -> Result<AnalysisResult, IrError> {
    Ok(Box::new(compute_escaped_symbols(context, &function)))
}

fn compute_escaped_symbols(context: &Context, function: &Function) -> EscapedSymbols {
    let add_from_val = |result: &mut FxHashSet<Symbol>, val: &Value, is_complete: &mut bool| {
        let (complete, syms) = get_referred_symbols(context, *val).consume();

        *is_complete &= complete;

        syms.iter().for_each(|s| {
            result.insert(*s);
        });
    };

    let mut result = FxHashSet::default();
    let mut is_complete = true;

    for (_block, inst) in function.instruction_iter(context) {
        match &inst.get_instruction(context).unwrap().op {
            InstOp::AsmBlock(_, args) => {
                for arg_init in args.iter().filter_map(|arg| arg.initializer) {
                    add_from_val(&mut result, &arg_init, &mut is_complete)
                }
            }
            InstOp::UnaryOp { .. } => (),
            InstOp::BinaryOp { .. } => (),
            InstOp::BitCast(_, _) => (),
            InstOp::Branch(_) => (),
            InstOp::Call(callee, args) => args
                .iter()
                .enumerate()
                .filter(|(arg_idx, _arg)| {
                    // Immutable arguments are not considered as escaping symbols.
                    !callee.is_arg_immutable(context, *arg_idx)
                })
                .for_each(|(_, v)| add_from_val(&mut result, v, &mut is_complete)),
            InstOp::CastPtr(ptr, _) => add_from_val(&mut result, ptr, &mut is_complete),
            InstOp::Cmp(_, _, _) => (),
            InstOp::ConditionalBranch { .. } => (),
            InstOp::ContractCall { params, .. } => {
                add_from_val(&mut result, params, &mut is_complete)
            }
            InstOp::FuelVm(_) => (),
            InstOp::GetLocal(_) => (),
            InstOp::GetGlobal(_) => (),
            InstOp::GetConfig(_, _) => (),
            InstOp::GetStorageKey(_) => (),
            InstOp::GetElemPtr { .. } => (),
            InstOp::IntToPtr(_, _) => (),
            InstOp::Load(_) => (),
            InstOp::MemCopyBytes { .. } => (),
            InstOp::MemCopyVal { .. } => (),
            InstOp::MemClearVal { .. } => (),
            InstOp::Nop => (),
            InstOp::PtrToInt(v, _) => add_from_val(&mut result, v, &mut is_complete),
            InstOp::Ret(_, _) => (),
            InstOp::Store { stored_val, .. } => {
                add_from_val(&mut result, stored_val, &mut is_complete)
            }
            InstOp::Alloc { .. } => (),
            InstOp::InitAggr(init_aggr) => {
                // Conceptually, we can think of `InitAggr` as a series of stores into the aggregate.
                // If any of the initializer values refer to symbols, those symbols escape.
                // This happens when the aggregate contains pointers, e.g., a Sway struct with fields
                // that are references.
                for init in init_aggr.initializers.iter() {
                    add_from_val(&mut result, init, &mut is_complete);
                }
            }
        }
    }

    if is_complete {
        EscapedSymbols::Complete(result)
    } else {
        EscapedSymbols::Incomplete(result)
    }
}

/// Pointers that may possibly be loaded from the instruction `inst`.
pub fn get_loaded_ptr_values(context: &Context, inst: Value) -> Vec<Value> {
    match &inst.get_instruction(context).unwrap().op {
        InstOp::UnaryOp { .. }
        | InstOp::BinaryOp { .. }
        | InstOp::BitCast(_, _)
        | InstOp::Branch(_)
        | InstOp::ConditionalBranch { .. }
        | InstOp::Cmp(_, _, _)
        | InstOp::Nop
        | InstOp::CastPtr(_, _)
        | InstOp::GetLocal(_)
        | InstOp::GetGlobal(_)
        | InstOp::GetConfig(_, _)
        | InstOp::GetStorageKey(_)
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
        InstOp::MemClearVal { .. } => vec![],
        InstOp::MemCopyBytes { src_val_ptr, .. }
        | InstOp::MemCopyVal { src_val_ptr, .. }
        | InstOp::Ret(src_val_ptr, _)
        | InstOp::Load(src_val_ptr)
        | InstOp::FuelVm(FuelVmInstruction::Log {
            log_val: src_val_ptr,
            ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateLoadWord {
            key: src_val_ptr, ..
        })
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
        InstOp::Alloc { .. } => vec![],
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
        // Similar like `Store` that never loads a pointer that might be its value,
        // `InitAggr` does not load any pointers it might have in the initializers.
        InstOp::InitAggr(_) => vec![],
    }
}

/// [Symbol]s that may possibly, directly or indirectly, be loaded from the instruction `inst`.
pub fn get_loaded_symbols(context: &Context, inst: Value) -> ReferredSymbols {
    let mut res = IndexSet::default();
    let mut is_complete = true;
    for val in get_loaded_ptr_values(context, inst) {
        let (complete, syms) = get_referred_symbols(context, val).consume();

        is_complete &= complete;

        for sym in syms {
            res.insert(sym);
        }
    }

    ReferredSymbols::new(is_complete, res)
}

/// Pointers that may possibly be stored to the instruction `inst`.
pub fn get_stored_ptr_values(context: &Context, inst: Value) -> Vec<Value> {
    match &inst.get_instruction(context).unwrap().op {
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
        | InstOp::GetGlobal(_)
        | InstOp::GetConfig(_, _)
        | InstOp::GetStorageKey(_)
        | InstOp::GetElemPtr { .. }
        | InstOp::IntToPtr(_, _) => vec![],
        InstOp::ContractCall { params, .. } => vec![*params],
        InstOp::Call(_, args) => args.clone(),
        InstOp::AsmBlock(_, args) => args.iter().filter_map(|val| val.initializer).collect(),
        InstOp::MemCopyBytes { dst_val_ptr, .. }
        | InstOp::MemCopyVal { dst_val_ptr, .. }
        | InstOp::MemClearVal { dst_val_ptr }
        | InstOp::Store { dst_val_ptr, .. } => vec![*dst_val_ptr],
        InstOp::InitAggr(InitAggr {
            aggr_ptr,
            initializers: _,
        }) => vec![*aggr_ptr],
        InstOp::Load(_) => vec![],
        InstOp::Alloc { .. } => vec![],
        InstOp::FuelVm(vmop) => match vmop {
            FuelVmInstruction::Gtf { .. }
            | FuelVmInstruction::Log { .. }
            | FuelVmInstruction::ReadRegister(_)
            | FuelVmInstruction::Revert(_)
            | FuelVmInstruction::JmpMem
            | FuelVmInstruction::Smo { .. }
            | FuelVmInstruction::StateClear { .. } => vec![],
            FuelVmInstruction::StateLoadQuadWord { load_val, .. } => vec![*load_val],
            FuelVmInstruction::StateLoadWord { .. } | FuelVmInstruction::StateStoreWord { .. } => {
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

/// [Symbol]s that may possibly, directly or indirectly, be stored to the instruction `inst`.
pub fn get_stored_symbols(context: &Context, inst: Value) -> ReferredSymbols {
    let mut res = IndexSet::default();
    let mut is_complete = true;
    for val in get_stored_ptr_values(context, inst) {
        let (complete, syms) = get_referred_symbols(context, val).consume();

        is_complete &= complete;

        for sym in syms {
            res.insert(sym);
        }
    }

    ReferredSymbols::new(is_complete, res)
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
