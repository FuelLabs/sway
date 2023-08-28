//! Scalar Replacement of Aggregates

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    combine_indices, compute_escaped_symbols, get_loaded_ptr_values, get_stored_ptr_values, get_symbols, AnalysisResults, Context,
    Function, Instruction, IrError, LocalVar, Pass, PassMutability, ScopedPass, Symbol, Type,
    Value,
};

pub const SROA_NAME: &str = "sroa";

pub fn create_sroa_pass() -> Pass {
    Pass {
        name: SROA_NAME,
        descr: "Scalar replacement of aggregates.",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(sroa)),
    }
}

// Split at a local aggregate variable into its constituent scalars.
// Returns a map from the offset of each scalar field to the new local created for it.
fn split_aggregate(
    context: &mut Context,
    function: Function,
    local_aggr: LocalVar,
) -> FxHashMap<u32, LocalVar> {
    let ty = local_aggr
        .get_type(context)
        .get_pointee_type(context)
        .expect("Local not a pointer");
    assert!(ty.is_aggregate(context));
    let mut res = FxHashMap::default();
    let aggr_base_name = function
        .lookup_local_name(context, &local_aggr)
        .cloned()
        .unwrap_or("".to_string());

    fn split_type(
        context: &mut Context,
        function: Function,
        aggr_base_name: &String,
        map: &mut FxHashMap<u32, LocalVar>,
        ty: Type,
        base_off: &mut u32,
    ) {
        if !super::target_fuel::is_demotable_type(context, &ty) {
            let ty_size: u32 = ty.size_in_bytes(context).try_into().unwrap();
            let name = aggr_base_name.clone() + &base_off.to_string();
            let scalarised_local = function.new_unique_local_var(context, name, ty, None, true);
            map.insert(*base_off, scalarised_local);
            *base_off += ty_size;
        } else {
            assert!(ty.is_aggregate(context));
            let mut i = 0;
            while let Some(member_ty) = ty.get_indexed_type(context, &[i]) {
                split_type(context, function, aggr_base_name, map, member_ty, base_off);
                i += 1;
            }
        }
    }

    let mut base_off = 0;
    split_type(
        context,
        function,
        &aggr_base_name,
        &mut res,
        ty,
        &mut base_off,
    );
    res
}

/// Promote aggregates to scalars, so that other optimizations
/// such as mem2reg can treat them as any other SSA value.
pub fn sroa(
    context: &mut Context,
    _analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let candidates = candidate_symbols(context, function);

    if candidates.is_empty() {
        return Ok(false);
    }
    // We now split each candidate into constituent scalar variables.
    let offset_scalar_map: FxHashMap<Symbol, FxHashMap<u32, LocalVar>> = candidates
        .iter()
        .map(|sym| {
            let Symbol::Local(local_aggr) = sym else { panic!("Expected only local candidates") };
            (*sym, split_aggregate(context, function, *local_aggr))
        })
        .collect();

    let mut scalar_replacements = FxHashMap::<Value, Value>::default();

    for block in function.block_iter(context) {
        let mut new_insts = Vec::new();
        for inst in block.instruction_iter(context) {
            let loaded_pointers = get_loaded_ptr_values(context, inst);
            let stored_pointers = get_stored_ptr_values(context, inst);

            for ptr in loaded_pointers.iter().chain(stored_pointers.iter()) {
                let syms = get_symbols(context, *ptr);
                if syms.len() == 1 && candidates.contains(&syms[0]) {
                    let Some(offset) =
                        combine_indices(context, *ptr).
                        and_then(|indices| 
                            syms[0]
                            .get_type(context)
                            .get_pointee_type(context)
                            .and_then(|pointee_ty| pointee_ty.get_value_indexed_offset(context, &indices)))
                        else { continue; };
                    let remapped_var = offset_scalar_map
                        .get(&syms[0])
                        .unwrap()
                        .get(&(offset as u32))
                        .unwrap();
                    let scalarized_local =
                        Value::new_instruction(context, Instruction::GetLocal(*remapped_var));
                    new_insts.push(scalarized_local);
                    scalar_replacements.insert(*ptr, scalarized_local);
                }
            }
            new_insts.push(inst);
        }
        context.blocks[block.0].instructions = new_insts;
    }

    function.replace_values(context, &scalar_replacements, None);

    Ok(true)
}

/// Only the following aggregates can be scalarised:
/// 1. Does not escape.
/// 2. Is always accessed via a scalar (register sized) field.
///    i.e., The entire aggregate or a sub-aggregate isn't loaded / stored.
fn candidate_symbols(context: &Context, function: Function) -> FxHashSet<Symbol> {
    let escaped_symbols = compute_escaped_symbols(context, &function);
    let mut candidates: FxHashSet<Symbol> = function
        .locals_iter(context)
        .filter_map(|(_, l)| {
            let sym = Symbol::Local(*l);
            (!escaped_symbols.contains(&sym)
                && l.get_type(context)
                    .get_pointee_type(context)
                    .is_some_and(|pointee_ty| pointee_ty.is_aggregate(context)))
            .then_some(sym)
        })
        .collect();

    // We walk the function to remove from `candidates`, any local that is
    // 1. accessed by a bigger-than-register sized load / store.
    // 2. OR accessed via a non-const indexing.
    // 3. OR aliased to a pointer that may point to more than one symbol.
    for (_, inst) in function.instruction_iter(context) {
        let loaded_pointers = get_loaded_ptr_values(context, inst);
        let stored_pointers = get_stored_ptr_values(context, inst);

        for ptr in loaded_pointers.iter().chain(stored_pointers.iter()) {
            let syms = get_symbols(context, *ptr);
            if syms.len() != 1 {
                for sym in &syms {
                    candidates.remove(sym);
                }
            }
            if combine_indices(context, *ptr).map_or(false, |indices| {
                indices.iter().any(|idx| !idx.is_constant(context))
            }) || ptr.match_ptr_type(context).is_some_and(|pointee_ty| {
                super::target_fuel::is_demotable_type(context, &pointee_ty)
            }) {
                candidates.remove(&syms[0]);
            }
        }
    }

    candidates
}
