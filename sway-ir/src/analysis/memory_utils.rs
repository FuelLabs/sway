use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    BlockArgument, Context, FuelVmInstruction, Function, Instruction, LocalVar, Type, Value,
    ValueDatum,
};

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
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

/// A value may (indirectly) refer to one or more symbols.
pub fn get_symbols(context: &Context, val: Value) -> Vec<Symbol> {
    let mut visited = FxHashSet::default();
    fn get_symbols_rec(
        context: &Context,
        visited: &mut FxHashSet<Value>,
        val: Value,
    ) -> Vec<Symbol> {
        if visited.contains(&val) {
            return vec![];
        }
        visited.insert(val);
        match context.values[val.0].value {
            ValueDatum::Instruction(Instruction::GetLocal(local)) => vec![Symbol::Local(local)],
            ValueDatum::Instruction(Instruction::GetElemPtr { base, .. }) => {
                get_symbols_rec(context, visited, base)
            }
            ValueDatum::Argument(b) => {
                if b.block.get_label(context) == "entry" {
                    vec![Symbol::Arg(b)]
                } else {
                    b.block
                        .pred_iter(context)
                        .map(|pred| b.get_val_coming_from(context, pred).unwrap())
                        .flat_map(|v| get_symbols_rec(context, visited, v))
                        .collect()
                }
            }
            _ => vec![],
        }
    }
    get_symbols_rec(context, &mut visited, val)
}

/// If [get_symbols] is a singleton, return that, otherwise None.
pub fn get_symbol(context: &Context, val: Value) -> Option<Symbol> {
    let syms = get_symbols(context, val);
    (syms.len() == 1).then(|| syms[0])
}

/// Combine a series of GEPs into one.
pub fn combine_indices(context: &Context, val: Value) -> Option<Vec<Value>> {
    match &context.values[val.0].value {
        ValueDatum::Instruction(Instruction::GetLocal(_)) => Some(vec![]),
        ValueDatum::Instruction(Instruction::GetElemPtr {
            base,
            elem_ptr_ty: _,
            indices,
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
                .get_indexed_offset(context, &combine_indices(context, val)?)?;
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

/// For a pointer Value, get the pointee type.
pub fn pointee_size(context: &Context, ptr_val: Value) -> u64 {
    ptr_val
        .get_type(context)
        .unwrap()
        .get_pointee_type(context)
        .expect("Expected arg to be a pointer")
        .size_in_bytes(context)
}

/// Get symbols loaded by instruction
pub fn get_loaded_symbols(context: &Context, inst: Value) -> Vec<Symbol> {
    match inst.get_instruction(context).unwrap() {
        Instruction::BinaryOp { .. }
        | Instruction::BitCast(_, _)
        | Instruction::Branch(_)
        | Instruction::ConditionalBranch { .. }
        | Instruction::Cmp(_, _, _)
        | Instruction::Nop
        | Instruction::CastPtr(_, _)
        | Instruction::GetLocal(_)
        | Instruction::GetElemPtr { .. }
        | Instruction::IntToPtr(_, _) => vec![],
        Instruction::PtrToInt(src_val_ptr, _) => get_symbols(context, *src_val_ptr).to_vec(),
        Instruction::ContractCall {
            params,
            coins,
            asset_id,
            ..
        } => vec![*params, *coins, *asset_id]
            .iter()
            .flat_map(|val| get_symbols(context, *val).to_vec())
            .collect(),
        Instruction::Call(_, args) => args
            .iter()
            .flat_map(|val| get_symbols(context, *val).to_vec())
            .collect(),
        Instruction::AsmBlock(_, args) => args
            .iter()
            .filter_map(|val| {
                val.initializer
                    .map(|val| get_symbols(context, val).to_vec())
            })
            .flatten()
            .collect(),
        Instruction::MemCopyBytes { src_val_ptr, .. }
        | Instruction::MemCopyVal { src_val_ptr, .. }
        | Instruction::Ret(src_val_ptr, _)
        | Instruction::Load(src_val_ptr)
        | Instruction::FuelVm(FuelVmInstruction::Log {
            log_val: src_val_ptr,
            ..
        })
        | Instruction::FuelVm(FuelVmInstruction::StateLoadWord(src_val_ptr))
        | Instruction::FuelVm(FuelVmInstruction::StateStoreWord {
            key: src_val_ptr, ..
        })
        | Instruction::FuelVm(FuelVmInstruction::StateLoadQuadWord {
            key: src_val_ptr, ..
        })
        | Instruction::FuelVm(FuelVmInstruction::StateClear {
            key: src_val_ptr, ..
        }) => get_symbols(context, *src_val_ptr).to_vec(),
        Instruction::FuelVm(FuelVmInstruction::StateStoreQuadWord {
            stored_val: memopd1,
            key: memopd2,
            ..
        })
        | Instruction::FuelVm(FuelVmInstruction::Smo {
            recipient: memopd1,
            message: memopd2,
            ..
        }) => get_symbols(context, *memopd1)
            .iter()
            .cloned()
            .chain(get_symbols(context, *memopd2).iter().cloned())
            .collect(),
        Instruction::Store { dst_val_ptr: _, .. } => vec![],
        Instruction::FuelVm(FuelVmInstruction::Gtf { .. })
        | Instruction::FuelVm(FuelVmInstruction::ReadRegister(_))
        | Instruction::FuelVm(FuelVmInstruction::Revert(_)) => vec![],
    }
}

/// Get symbols stored to by instruction
pub fn get_stored_symbols(context: &Context, inst: Value) -> Vec<Symbol> {
    match inst.get_instruction(context).unwrap() {
        Instruction::BinaryOp { .. }
        | Instruction::BitCast(_, _)
        | Instruction::Branch(_)
        | Instruction::ConditionalBranch { .. }
        | Instruction::Cmp(_, _, _)
        | Instruction::Nop
        | Instruction::PtrToInt(_, _)
        | Instruction::Ret(_, _)
        | Instruction::CastPtr(_, _)
        | Instruction::GetLocal(_)
        | Instruction::GetElemPtr { .. }
        | Instruction::IntToPtr(_, _) => vec![],
        Instruction::ContractCall { params, .. } => get_symbols(context, *params),
        Instruction::Call(_, args) => args
            .iter()
            .flat_map(|val| get_symbols(context, *val).to_vec())
            .collect(),
        Instruction::AsmBlock(_, args) => args
            .iter()
            .filter_map(|val| {
                val.initializer
                    .map(|val| get_symbols(context, val).to_vec())
            })
            .flatten()
            .collect(),
        Instruction::MemCopyBytes { dst_val_ptr, .. }
        | Instruction::MemCopyVal { dst_val_ptr, .. }
        | Instruction::Store { dst_val_ptr, .. } => get_symbols(context, *dst_val_ptr).to_vec(),
        Instruction::Load(_) => vec![],
        Instruction::FuelVm(vmop) => match vmop {
            FuelVmInstruction::Gtf { .. }
            | FuelVmInstruction::Log { .. }
            | FuelVmInstruction::ReadRegister(_)
            | FuelVmInstruction::Revert(_)
            | FuelVmInstruction::Smo { .. }
            | FuelVmInstruction::StateClear { .. } => vec![],
            FuelVmInstruction::StateLoadQuadWord { load_val, .. } => {
                get_symbols(context, *load_val).to_vec()
            }
            FuelVmInstruction::StateLoadWord(_) | FuelVmInstruction::StateStoreWord { .. } => {
                vec![]
            }
            FuelVmInstruction::StateStoreQuadWord { stored_val: _, .. } => vec![],
        },
    }
}
