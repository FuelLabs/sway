//! Scalar Replacement of Aggregates

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    combine_indices, compute_escaped_symbols, get_gep_referred_symbols, get_loaded_ptr_values,
    get_stored_ptr_values, pointee_size, AnalysisResults, Constant, ConstantContent, ConstantValue,
    Context, EscapedSymbols, Function, InstOp, IrError, LocalVar, Pass, PassMutability, ScopedPass,
    Symbol, Type, Value,
};

pub const SROA_NAME: &str = "sroa";

pub fn create_sroa_pass() -> Pass {
    Pass {
        name: SROA_NAME,
        descr: "Scalar replacement of aggregates",
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
        initializer: Option<Constant>,
        base_off: &mut u32,
    ) {
        fn constant_index(context: &mut Context, c: &Constant, idx: usize) -> Constant {
            match &c.get_content(context).value {
                ConstantValue::Array(cs) | ConstantValue::Struct(cs) => Constant::unique(
                    context,
                    cs.get(idx)
                        .expect("Malformed initializer. Cannot index into sub-initializer")
                        .clone(),
                ),
                _ => panic!("Expected only array or struct const initializers"),
            }
        }
        if !super::target_fuel::is_demotable_type(context, &ty) {
            let ty_size: u32 = ty.size(context).in_bytes().try_into().unwrap();
            let name = aggr_base_name.clone() + &base_off.to_string();
            let scalarised_local =
                function.new_unique_local_var(context, name, ty, initializer, false);
            map.insert(*base_off, scalarised_local);

            *base_off += ty_size;
        } else {
            let mut i = 0;
            while let Some(member_ty) = ty.get_indexed_type(context, &[i]) {
                let initializer = initializer
                    .as_ref()
                    .map(|c| constant_index(context, c, i as usize));
                split_type(
                    context,
                    function,
                    aggr_base_name,
                    map,
                    member_ty,
                    initializer,
                    base_off,
                );

                if ty.is_struct(context) {
                    *base_off = crate::size_bytes_round_up_to_word_alignment!(*base_off);
                }

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
        local_aggr.get_initializer(context).cloned(),
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
            let Symbol::Local(local_aggr) = sym else {
                panic!("Expected only local candidates")
            };
            (*sym, split_aggregate(context, function, *local_aggr))
        })
        .collect();

    let mut scalar_replacements = FxHashMap::<Value, Value>::default();

    for block in function.block_iter(context) {
        let mut new_insts = Vec::new();
        for inst in block.instruction_iter(context) {
            if let InstOp::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } = inst.get_instruction(context).unwrap().op
            {
                let src_syms = get_gep_referred_symbols(context, src_val_ptr);
                let dst_syms = get_gep_referred_symbols(context, dst_val_ptr);

                // If neither source nor dest needs rewriting, we skip.
                let src_sym = src_syms
                    .iter()
                    .next()
                    .filter(|src_sym| candidates.contains(src_sym));
                let dst_sym = dst_syms
                    .iter()
                    .next()
                    .filter(|dst_sym| candidates.contains(dst_sym));
                if src_sym.is_none() && dst_sym.is_none() {
                    new_insts.push(inst);
                    continue;
                }

                struct ElmDetail {
                    offset: u32,
                    r#type: Type,
                    indices: Vec<u32>,
                }

                // compute the offsets at which each (nested) field in our pointee type is at.
                fn calc_elm_details(
                    context: &Context,
                    details: &mut Vec<ElmDetail>,
                    ty: Type,
                    base_off: &mut u32,
                    base_index: &mut Vec<u32>,
                ) {
                    if !super::target_fuel::is_demotable_type(context, &ty) {
                        let ty_size: u32 = ty.size(context).in_bytes().try_into().unwrap();
                        details.push(ElmDetail {
                            offset: *base_off,
                            r#type: ty,
                            indices: base_index.clone(),
                        });
                        *base_off += ty_size;
                    } else {
                        assert!(ty.is_aggregate(context));
                        base_index.push(0);
                        let mut i = 0;
                        while let Some(member_ty) = ty.get_indexed_type(context, &[i]) {
                            calc_elm_details(context, details, member_ty, base_off, base_index);
                            i += 1;
                            *base_index.last_mut().unwrap() += 1;

                            if ty.is_struct(context) {
                                *base_off =
                                    crate::size_bytes_round_up_to_word_alignment!(*base_off);
                            }
                        }
                        base_index.pop();
                    }
                }
                let mut local_base_offset = 0;
                let mut local_base_index = vec![];
                let mut elm_details = vec![];
                calc_elm_details(
                    context,
                    &mut elm_details,
                    src_val_ptr
                        .get_type(context)
                        .unwrap()
                        .get_pointee_type(context)
                        .expect("Unable to determine pointee type of pointer"),
                    &mut local_base_offset,
                    &mut local_base_index,
                );

                // Handle the source pointer first.
                let mut elm_local_map = FxHashMap::default();
                if let Some(src_sym) = src_sym {
                    // The source symbol is a candidate. So it has been split into scalars.
                    // Load each of these into a SSA variable.
                    let base_offset = combine_indices(context, src_val_ptr)
                        .and_then(|indices| {
                            src_sym
                                .get_type(context)
                                .get_pointee_type(context)
                                .and_then(|pointee_ty| {
                                    pointee_ty.get_value_indexed_offset(context, &indices)
                                })
                        })
                        .expect("Source of memcpy was incorrectly identified as a candidate.")
                        as u32;
                    for detail in elm_details.iter() {
                        let elm_offset = detail.offset;
                        let actual_offset = elm_offset + base_offset;
                        let remapped_var = offset_scalar_map
                            .get(src_sym)
                            .unwrap()
                            .get(&actual_offset)
                            .unwrap();
                        let scalarized_local =
                            Value::new_instruction(context, block, InstOp::GetLocal(*remapped_var));
                        let load =
                            Value::new_instruction(context, block, InstOp::Load(scalarized_local));
                        elm_local_map.insert(elm_offset, load);
                        new_insts.push(scalarized_local);
                        new_insts.push(load);
                    }
                } else {
                    // The source symbol is not a candidate. So it won't be split into scalars.
                    // We must use GEPs to load each individual element into an SSA variable.
                    for ElmDetail {
                        offset,
                        r#type,
                        indices,
                    } in &elm_details
                    {
                        let elm_index_values = indices
                            .iter()
                            .map(|&index| {
                                let c = ConstantContent::new_uint(context, 64, index.into());
                                let c = Constant::unique(context, c);
                                Value::new_constant(context, c)
                            })
                            .collect();
                        let elem_ptr_ty = Type::new_typed_pointer(context, *r#type);
                        let elm_addr = Value::new_instruction(
                            context,
                            block,
                            InstOp::GetElemPtr {
                                base: src_val_ptr,
                                elem_ptr_ty,
                                indices: elm_index_values,
                            },
                        );
                        let load = Value::new_instruction(context, block, InstOp::Load(elm_addr));
                        elm_local_map.insert(*offset, load);
                        new_insts.push(elm_addr);
                        new_insts.push(load);
                    }
                }
                if let Some(dst_sym) = dst_sym {
                    // The dst symbol is a candidate. So it has been split into scalars.
                    // Store to each of these from the SSA variable we created above.
                    let base_offset = combine_indices(context, dst_val_ptr)
                        .and_then(|indices| {
                            dst_sym
                                .get_type(context)
                                .get_pointee_type(context)
                                .and_then(|pointee_ty| {
                                    pointee_ty.get_value_indexed_offset(context, &indices)
                                })
                        })
                        .expect("Source of memcpy was incorrectly identified as a candidate.")
                        as u32;
                    for detail in elm_details.iter() {
                        let elm_offset = detail.offset;
                        let actual_offset = elm_offset + base_offset;
                        let remapped_var = offset_scalar_map
                            .get(dst_sym)
                            .unwrap()
                            .get(&actual_offset)
                            .unwrap();
                        let scalarized_local =
                            Value::new_instruction(context, block, InstOp::GetLocal(*remapped_var));
                        let loaded_source = elm_local_map
                            .get(&elm_offset)
                            .expect("memcpy source not loaded");
                        let store = Value::new_instruction(
                            context,
                            block,
                            InstOp::Store {
                                dst_val_ptr: scalarized_local,
                                stored_val: *loaded_source,
                            },
                        );
                        new_insts.push(scalarized_local);
                        new_insts.push(store);
                    }
                } else {
                    // The dst symbol is not a candidate. So it won't be split into scalars.
                    // We must use GEPs to store to each individual element from its SSA variable.
                    for ElmDetail {
                        offset,
                        r#type,
                        indices,
                    } in elm_details
                    {
                        let elm_index_values = indices
                            .iter()
                            .map(|&index| {
                                let c = ConstantContent::new_uint(context, 64, index.into());
                                let c = Constant::unique(context, c);
                                Value::new_constant(context, c)
                            })
                            .collect();
                        let elem_ptr_ty = Type::new_typed_pointer(context, r#type);
                        let elm_addr = Value::new_instruction(
                            context,
                            block,
                            InstOp::GetElemPtr {
                                base: dst_val_ptr,
                                elem_ptr_ty,
                                indices: elm_index_values,
                            },
                        );
                        let loaded_source = elm_local_map
                            .get(&offset)
                            .expect("memcpy source not loaded");
                        let store = Value::new_instruction(
                            context,
                            block,
                            InstOp::Store {
                                dst_val_ptr: elm_addr,
                                stored_val: *loaded_source,
                            },
                        );
                        new_insts.push(elm_addr);
                        new_insts.push(store);
                    }
                }

                // We've handled the memcpy. it's been replaced with other instructions.
                continue;
            }
            let loaded_pointers = get_loaded_ptr_values(context, inst);
            let stored_pointers = get_stored_ptr_values(context, inst);

            for ptr in loaded_pointers.iter().chain(stored_pointers.iter()) {
                let syms = get_gep_referred_symbols(context, *ptr);
                if let Some(sym) = syms
                    .iter()
                    .next()
                    .filter(|sym| syms.len() == 1 && candidates.contains(sym))
                {
                    let Some(offset) = combine_indices(context, *ptr).and_then(|indices| {
                        sym.get_type(context)
                            .get_pointee_type(context)
                            .and_then(|pointee_ty| {
                                pointee_ty.get_value_indexed_offset(context, &indices)
                            })
                    }) else {
                        continue;
                    };
                    let remapped_var = offset_scalar_map
                        .get(sym)
                        .unwrap()
                        .get(&(offset as u32))
                        .unwrap();
                    let scalarized_local =
                        Value::new_instruction(context, block, InstOp::GetLocal(*remapped_var));
                    new_insts.push(scalarized_local);
                    scalar_replacements.insert(*ptr, scalarized_local);
                }
            }
            new_insts.push(inst);
        }
        block.take_body(context, new_insts);
    }

    function.replace_values(context, &scalar_replacements, None);

    Ok(true)
}

// Is the aggregate type something that we can handle?
fn is_processable_aggregate(context: &Context, ty: Type) -> bool {
    fn check_sub_types(context: &Context, ty: Type) -> bool {
        match ty.get_content(context) {
            crate::TypeContent::Unit => true,
            crate::TypeContent::Bool => true,
            crate::TypeContent::Uint(width) => *width <= 64,
            crate::TypeContent::B256 => false,
            crate::TypeContent::Array(elm_ty, _) => check_sub_types(context, *elm_ty),
            crate::TypeContent::Union(_) => false,
            crate::TypeContent::Struct(fields) => {
                fields.iter().all(|ty| check_sub_types(context, *ty))
            }
            crate::TypeContent::Slice => false,
            crate::TypeContent::TypedSlice(..) => false,
            crate::TypeContent::Pointer => true,
            crate::TypeContent::TypedPointer(_) => true,
            crate::TypeContent::StringSlice => false,
            crate::TypeContent::StringArray(_) => false,
            crate::TypeContent::Never => false,
        }
    }
    ty.is_aggregate(context) && check_sub_types(context, ty)
}

// Filter out candidates that may not be profitable to scalarise.
// This can be tuned in detail in the future when we have real benchmarks.
fn profitability(context: &Context, function: Function, candidates: &mut FxHashSet<Symbol>) {
    // If a candidate is sufficiently big and there's at least one memcpy
    // accessing a big part of it, it may not be wise to scalarise it.
    for (_, inst) in function.instruction_iter(context) {
        if let InstOp::MemCopyVal {
            dst_val_ptr,
            src_val_ptr,
        } = inst.get_instruction(context).unwrap().op
        {
            if pointee_size(context, dst_val_ptr) > 200 {
                for sym in get_gep_referred_symbols(context, dst_val_ptr)
                    .union(&get_gep_referred_symbols(context, src_val_ptr))
                {
                    candidates.remove(sym);
                }
            }
        }
    }
}

/// Only the following aggregates can be scalarised:
/// 1. Does not escape.
/// 2. Is always accessed via a scalar (register sized) field.
///    i.e., The entire aggregate or a sub-aggregate isn't loaded / stored.
///    (with an exception of `mem_copy_val` which we can handle).
/// 3. Never accessed via non-const indexing.
/// 4. Not aliased via a pointer that may point to more than one symbol.
fn candidate_symbols(context: &Context, function: Function) -> FxHashSet<Symbol> {
    let escaped_symbols = match compute_escaped_symbols(context, &function) {
        EscapedSymbols::Complete(syms) => syms,
        EscapedSymbols::Incomplete(_) => return FxHashSet::<_>::default(),
    };

    let mut candidates: FxHashSet<Symbol> = function
        .locals_iter(context)
        .filter_map(|(_, l)| {
            let sym = Symbol::Local(*l);
            (!escaped_symbols.contains(&sym)
                && l.get_type(context)
                    .get_pointee_type(context)
                    .is_some_and(|pointee_ty| is_processable_aggregate(context, pointee_ty)))
            .then_some(sym)
        })
        .collect();

    // We walk the function to remove from `candidates`, any local that is
    // 1. accessed by a bigger-than-register sized load / store.
    //    (we make an exception for load / store in `mem_copy_val` as that can be handled).
    // 2. OR accessed via a non-const indexing.
    // 3. OR aliased to a pointer that may point to more than one symbol.
    for (_, inst) in function.instruction_iter(context) {
        let loaded_pointers = get_loaded_ptr_values(context, inst);
        let stored_pointers = get_stored_ptr_values(context, inst);

        let inst = inst.get_instruction(context).unwrap();
        for ptr in loaded_pointers.iter().chain(stored_pointers.iter()) {
            let syms = get_gep_referred_symbols(context, *ptr);
            if syms.len() != 1 {
                for sym in &syms {
                    candidates.remove(sym);
                }
                continue;
            }
            if combine_indices(context, *ptr)
                .is_some_and(|indices| indices.iter().any(|idx| !idx.is_constant(context)))
                || ptr.match_ptr_type(context).is_some_and(|pointee_ty| {
                    super::target_fuel::is_demotable_type(context, &pointee_ty)
                        && !matches!(inst.op, InstOp::MemCopyVal { .. })
                })
            {
                candidates.remove(syms.iter().next().unwrap());
            }
        }
    }

    profitability(context, function, &mut candidates);

    candidates
}
