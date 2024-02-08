//! An analysis to compute symbols that escape out from a function.
//! This could be into another function, or via ptr_to_int etc.
//! Any transformations involving such symbols are unsafe.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    AnalysisResult, AnalysisResultT, AnalysisResults, BlockArgument, Context, FuelVmInstruction,
    Function, InstOp, Instruction, IrError, LocalVar, Pass, PassMutability, ScopedPass, Type,
    Value, ValueDatum,
};

pub const ESCAPED_SYMBOLS_NAME: &str = "escaped_symbols";

pub fn create_escaped_symbols_pass() -> Pass {
    Pass {
        name: ESCAPED_SYMBOLS_NAME,
        descr: "Symbols that escape / cannot be analysed",
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

// A value may (indirectly) refer to one or more symbols.
pub fn get_symbols(context: &Context, val: Value) -> FxHashSet<Symbol> {
    let mut visited = FxHashSet::default();
    let mut symbols = FxHashSet::<Symbol>::default();
    fn get_symbols_rec(
        context: &Context,
        symbols: &mut FxHashSet<Symbol>,
        visited: &mut FxHashSet<Value>,
        val: Value,
    ) {
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
            }) => get_symbols_rec(context, symbols, visited, base),
            ValueDatum::Argument(b) => {
                if b.block.get_label(context) == "entry" {
                    symbols.insert(Symbol::Arg(b));
                } else {
                    b.block
                        .pred_iter(context)
                        .map(|pred| b.get_val_coming_from(context, pred).unwrap())
                        .for_each(|v| get_symbols_rec(context, symbols, visited, v))
                }
            }
            _ => (),
        }
    }
    get_symbols_rec(context, &mut symbols, &mut visited, val);
    symbols
}

pub fn get_symbol(context: &Context, val: Value) -> Option<Symbol> {
    let syms = get_symbols(context, val);
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
        get_symbols(context, *val).iter().for_each(|s| {
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
        | InstOp::FuelVm(FuelVmInstruction::Revert(_) | FuelVmInstruction::JmpToSsp) => vec![],
        InstOp::FuelVm(FuelVmInstruction::WideUnaryOp { arg, .. }) => vec![*arg],
        InstOp::FuelVm(FuelVmInstruction::WideBinaryOp { arg1, arg2, .. })
        | InstOp::FuelVm(FuelVmInstruction::WideCmpOp { arg1, arg2, .. }) => {
            vec![*arg1, *arg2]
        }
        InstOp::FuelVm(FuelVmInstruction::WideModularOp {
            arg1, arg2, arg3, ..
        }) => vec![*arg1, *arg2, *arg3],
    }
}

/// Symbols that may possibly be loaded from.
pub fn get_loaded_symbols(context: &Context, val: Value) -> FxHashSet<Symbol> {
    let mut res = FxHashSet::default();
    for val in get_loaded_ptr_values(context, val) {
        for sym in get_symbols(context, val) {
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
            | FuelVmInstruction::JmpToSsp
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
        },
    }
}

/// Symbols that may possibly be stored to.
pub fn get_stored_symbols(context: &Context, val: Value) -> FxHashSet<Symbol> {
    let mut res = FxHashSet::default();
    for val in get_stored_ptr_values(context, val) {
        for sym in get_symbols(context, val) {
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
/// for each symbol that this may alias to.
pub fn get_memory_offsets(context: &Context, val: Value) -> FxHashMap<Symbol, u64> {
    get_symbols(context, val)
        .into_iter()
        .filter_map(|sym| {
            let offset = sym
                .get_type(context)
                .get_pointee_type(context)?
                .get_value_indexed_offset(context, &combine_indices(context, val)?)?;
            Some((sym, offset))
        })
        .collect()
}

/// Can memory ranges [val1, val1+len1] and [val2, val2+len2] overlap?
/// Conservatively returns true if cannot statically determine.
pub fn may_alias(context: &Context, val1: Value, len1: u64, val2: Value, len2: u64) -> bool {
    let mem_offsets_1 = get_memory_offsets(context, val1);
    let mem_offsets_2 = get_memory_offsets(context, val2);

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
    let mem_offsets_1 = get_memory_offsets(context, val1);
    let mem_offsets_2 = get_memory_offsets(context, val2);

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
