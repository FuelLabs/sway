//! Lowering of the `init_aggr` instruction.
//!
//! The lowering replaces `init_aggr` with an optimal sequence of
//! instructions like `store`, `mem_clear_val`, `mem_copy_val`, etc.

use std::{collections::HashSet, vec};

use rustc_hash::FxHashMap;

use crate::{
    dominator::{self},
    AnalysisResults, BinaryOpKind, Context, Function, InitAggrInitializer,
    InstOp, Instruction, InstructionInserter, IrError, MetadataIndex, Pass, PassMutability,
    Predicate, ScopedPass, Type, TypeContent, Value,
};

pub const INIT_AGGR_LOWERING_NAME: &str = "lower-init-aggr";

pub fn create_init_aggr_lowering_pass() -> Pass {
    Pass {
        name: INIT_AGGR_LOWERING_NAME,
        descr: "Lowering of `init_aggr` instructions",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(init_aggr_lowering)),
    }
}

pub fn init_aggr_lowering<'a, 'b>(
    context: &'a mut Context<'b>,
    _analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let root_init_aggrs = find_root_init_aggrs(context, function);
    if root_init_aggrs.is_empty() {
        return Ok(false);
    }

    // Lower each `root_init_aggr` in a most optimized way.
    //
    // Lowering removes only nested `init_aggr` instructions,
    // but does not remove the `root_init_aggr` instruction yet, nor replaces its uses.
    // This is done after all lowerings are complete, and for that we build the `replace_map`
    // which maps each `root_init_aggr` to the aggregate that it initializes.
    let mut replace_map = FxHashMap::<Value, Value>::default();
    for root_init_aggr in root_init_aggrs.iter() {
        let (root_aggr_ptr, initializers) = deconstruct_init_aggr(context, *root_init_aggr);

        replace_map.insert(*root_init_aggr, root_aggr_ptr);

        let root_aggr_type = root_aggr_ptr
            .match_ptr_type(context)
            .expect("`root_aggr_ptr` must be a pointer");

        let _ = lower_mostly_zeroed_aggregate(
            context,
            *root_init_aggr,
            root_aggr_type,
            root_aggr_ptr,
            &initializers,
        ) || lower_to_stores(
            context,
            *root_init_aggr,
            root_aggr_type,
            root_aggr_ptr,
            &mut Vec::new(),
            &initializers,
            false,
        );
    }

    // Replace all usages of `root_init_aggr`s with the pointers to the aggregates they initialize.
    function.replace_values(context, &replace_map, None);

    // Finally, remove all root `root_init_aggr` instructions.
    function.remove_instructions(context, |inst| root_init_aggrs.contains(&inst));

    // Check that all of the nested `init_aggr` instructions are removed.
    // TODO: This is a full scan of all instructions in almost all functions.
    //       If needed, we can improve this by scanning only the blocks that
    //       contained root `init_aggr` instructions, and there also to scan
    //       only up to the latest root `init_aggr`. But it seams to be a
    //       premature optimization.
    if function.instruction_iter(context).any(|(_block, inst)| matches!(inst.get_instruction(context).map(|inst| &inst.op), Some(InstOp::InitAggr(_)))) {
        return Err(IrError::InitAggrsNotLowered());
    }

    Ok(true)
}

/// Deconstructs `init_aggr` into `aggr_ptr` and `initializers`.
fn deconstruct_init_aggr(context: &Context, init_aggr: Value) -> (Value, Vec<InitAggrInitializer>) {
    let Some(Instruction {
        parent: _,
        op: InstOp::InitAggr(init_aggr),
    }) = init_aggr.get_instruction(context).cloned()
    else {
        panic!("`init_aggr` must be an `Instruction` with `op` of variant `InstOp::InitAggr`");
    };

    (
        init_aggr.aggr_ptr,
        init_aggr.initializers(context).collect(),
    )
}

/// This lowering checks whether a **root aggregate** being initialized is mostly zeroed,
/// i.e., whether most of its fields are initialized to zero values.
/// If so, it lowers the `init_aggr` to a `mem_clear_val` for the entire root aggregate,
/// followed by `store`s for the non-zero fields.
///
/// E.g., a very common case is initializing tuples like `(0, 0, 0, some_variable)`.
///
/// Note that this lowering is not recursive. It is always called with a **root aggregate**.
///
/// Returns `true` if the lowering was performed, `false` otherwise.
fn lower_mostly_zeroed_aggregate<'a, 'b>(
    context: &'a mut Context<'b>,
    root_init_aggr: Value,
    root_aggr_type: Type,
    root_aggr_ptr: Value,
    initializers: &[InitAggrInitializer],
) -> bool {
    /// Computes, recursively, the total size in bytes of zero-initialized
    /// fields within the `type_content`.
    ///
    /// The `type_size` is the size of the `type_content` potentially aligned
    /// within a parent. E.g., an `u8` element inside of an array has size of one byte,
    /// but within a struct, size of 8 bytes, because of alignment to word boundary.
    /// That's why `type_size` cannot be calculated internally within the function
    /// but **must be passed from the parent**.
    ///
    /// The top-level call will always pass a root aggregate `type_content`
    /// that is initialized with an `init_aggr` (array or struct).
    ///
    /// The recursion ends when we reach leaf non-aggregate types whose `type_size`
    /// is passed from the parent aggregate.
    ///
    /// Note that:
    /// - computing size of zeroed elements and comparing it to the full aggregate size,
    ///   **is only an approximate heuristics** for actually counting `store`s and `memcopy`s
    ///   needed to initialize the aggregate and comparing the savings when using `memclear`.
    /// - we could have also chosen to count only the size of leaf non-aggregates, regardless
    ///   of them being potentially aligned within a struct parent. This would require
    ///   also calculating the size of the root aggregate regardless of any alignments and
    ///   comparing against that. This way of counting would lead to different tradeoffs and
    ///   inaccuracies in the heuristics. E.g., the current way of counting sizes gives
    ///   different ratios in the edge case example of comparing `(0u8, 42u64)` and `([0u8], 42u64)`.
    ///   In ideal counting of `store`s these two cases should be the same.
    ///   Counting sized of only leaf non-aggregates leads to different tradeoffs, e.g.,
    ///   when having zeroed embedded aggregates that are initialized as [InitAggrInitializer::Value].
    ///   The chosen approach shows slight empirical advantage in having less regressions.
    // TODO: (INIT-AGGR) Can we improve heuristics here and actually count operations needed
    //       to initialize an aggregate? It should be counting ASM than, including knowing the
    //       gas cost. Too complex?
    fn compute_size_of_zeroed_elements(
        context: &Context,
        type_content: &TypeContent,
        type_size: u64,
        initializers: &[InitAggrInitializer],
    ) -> u64 {
        // TODO-MEMLAY: Warning! Here we make an assumption about the memory layout of
        //              structs and arrays.
        //              The memory layout of structs and arrays can be changed in the future.
        match type_content {
            TypeContent::Array(elem_type, length) => {
                assert_eq!(
                    *length as usize,
                    initializers.len(),
                    "`init_aggr` initializers must match the length of the array type"
                );

                // Array elements are packed. The size of the element is it's size in bytes.
                let elem_size = elem_type.size(context).in_bytes();

                let mut zero_size = 0u64;
                for init in initializers.into_iter() {
                    let init_values = match init {
                        InitAggrInitializer::Value(_) => {
                            // Not that this also deliberately includes enums.
                            if elem_type.is_aggregate(context) {
                                // Element type is an aggregate not initialized with `init_aggr`.
                                // We cannot inspect its content further in detail, but if we know
                                // it is runtime-zeroed, we can add it to the cumulative `zero_size`.
                                // If an aggregate is runtime-zeroed 
                                if init.is_runtime_zeroed(context) {
                                    zero_size += elem_size;
                                }
                                // Don't analyze this initializer further.
                                continue;
                            }

                            // This is non-aggregate leaf. Pass it down as the final recursive step.
                            vec![init.clone()]
                        }
                        InitAggrInitializer::NestedInitAggr {
                            load: _,
                            init_aggr: nested_init_aggr,
                        } => {
                            let (_nested_aggr_ptr, nested_ia_initializers) =
                                deconstruct_init_aggr(context, *nested_init_aggr);
                            // Pass the initializers down the recursive step.
                            nested_ia_initializers
                        }
                    };

                    let elem_zero_size =
                        compute_size_of_zeroed_elements(context, elem_type.get_content(context), elem_size, &init_values);

                    zero_size += elem_zero_size;
                }

                zero_size
            }
            TypeContent::Struct(field_types) => {
                assert_eq!(
                    field_types.len(),
                    initializers.len(),
                    "`init_aggr` initializers must match the number of fields in the struct type"
                );

                let mut zero_size = 0u64;
                for (init, field_type) in initializers.into_iter().zip(field_types) {
                    // Struct fields are aligned to word boundary.
                    let field_size = field_type.size(context).in_bytes_aligned();

                    let init_values = match init {
                        InitAggrInitializer::Value(_) => {
                            // Not that this also deliberately includes enums.
                            if field_type.is_aggregate(context) {
                                // Element type is an aggregate not initialized with `init_aggr`.
                                // We cannot inspect its content further in detail, but if we know
                                // it is runtime-zeroed, we can add it to the cumulative `zero_size`.
                                // If an aggregate is runtime-zeroed 
                                if init.is_runtime_zeroed(context) {
                                    zero_size += field_size;
                                }
                                // Don't analyze this initializer further.
                                continue;
                            }

                            // This is non-aggregate leaf. Pass it down as the final recursive step.
                            vec![init.clone()]
                        }
                        InitAggrInitializer::NestedInitAggr {
                            load: _,
                            init_aggr: nested_init_aggr,
                        } => {
                            let (_nested_aggr_ptr, nested_ia_initializers) =
                                deconstruct_init_aggr(context, *nested_init_aggr);
                            nested_ia_initializers
                        }
                    };

                    let field_zero_size =
                        compute_size_of_zeroed_elements(context, field_type.get_content(context), field_size, &init_values);

                    zero_size += field_zero_size;
                }

                zero_size
            }
            _ => {
                assert!(
                    matches!(initializers, [InitAggrInitializer::Value(_)]),
                    "a leaf element must be a non-aggregate with a single initializer of variant `InitAggrInitializer::Value`"
                );

                let is_zero_init = initializers[0].is_runtime_zeroed(context);
                let zero_size = if is_zero_init { type_size } else { 0 };
                zero_size
            }
        }
    }

    // The root aggregate is never embedded inside of any other aggregates.
    // It's total size is always its size in bytes.
    let total_size = root_aggr_type.size(context).in_bytes();
    let zero_size = compute_size_of_zeroed_elements(context, root_aggr_type.get_content(context), total_size, initializers);

    let zero_ratio = zero_size as f64 / total_size as f64;

    // Not mostly zeroed.
    if zero_ratio < 0.30 {
        return false;
    }

    // `lower_single_initializer_to_stores` stores values directly into the root aggregate,
    // so we need to make sure that the `mem_clear_val` is done before any of those stores.
    // We need to get the position of the outmost `init_aggr` which can be the root itself
    // iff it has no nested `init_aggr`s, or its outmost nested `init_aggr`.

    // 1. Collect all `init_aggr` related to the root, including the root itself.
    fn collect_init_aggrs<'a, 'b>(
        context: &'a Context<'b>,
        init_aggr: Value,
        init_aggrs: &mut FxHashMap<Value, usize>,
    ) {
        // All `init_aggr`s are initially marked with index 0 (unknown).
        init_aggrs.insert(init_aggr, 0);
        let Some(Instruction {
            parent: _,
            op: InstOp::InitAggr(init_aggr),
        }) = init_aggr.get_instruction(context)
        else {
            unreachable!("`init_aggr` is an `InstOp::InitAggr`");
        };
        for initializer in init_aggr.initializers(context) {
            if let InitAggrInitializer::NestedInitAggr {
                load: _,
                init_aggr: nested_init_aggr,
            } = initializer
            {
                collect_init_aggrs(context, nested_init_aggr, init_aggrs);
            }
        }
    }

    let mut init_aggrs = FxHashMap::<Value, usize>::default();
    collect_init_aggrs(context, root_init_aggr, &mut init_aggrs);

    // 2. Get the actual indices of `init_aggr`s increased by 1 so that 0 remains as "unknown".
    //    "Unknown" we can have only if a particular nested `init_aggr` is not in the same
    //    block as root `init_aggregate` which can never be the case.
    let parent_block = root_init_aggr
        .get_parent_block(context)
        .expect("`init_aggr` is an instruction and must have a parent block");
    for (inst_idx, inst) in parent_block.instruction_iter(context).enumerate() {
        if init_aggrs.contains_key(&inst) {
            init_aggrs.insert(inst, inst_idx + 1);
        }
    }

    // 3. Find the earliest instruction index among those `init_aggr`s.
    let (earliest_aggr_init_inst, earliest_aggr_init_inst_idx)  = init_aggrs
        .iter()
        .min_by(|(_inst1, index1), (_inst2, index2)| index1.cmp(index2))
        .map(|(inst, index)| (*inst, *index))
        .expect("there must be at least one `init_aggr` because we included the root `init_aggr`");

    // We do not consider cases like, e.g., this one, as nested `init_aggr`s:
    //   S {
    //      x: if condition {
    //          (0, 1)
    //      } else {
    //          (1, 0)
    //      }
    //   }
    // All nested `init_aggr`s of a particular root will always be in the same
    // block as the root. Let's conveniently assert that expectation here,
    // because we have an easy way to do it.
    assert_ne!(earliest_aggr_init_inst_idx, 0, "all nested `init_aggr`s must be in the same block as their root");

    // Perform the lowering:
    // 1. `mem_clear_val` for the entire aggregate.
    let inserter = get_inst_inserter_for_before_init_aggr(context, earliest_aggr_init_inst);
    inserter
        .mem_clear_val(root_aggr_ptr)
        .add_metadatum(context, root_init_aggr.get_metadata(context));

    // 2. `store`s for the non-zero fields.
    lower_to_stores(
        context,
        root_init_aggr,
        root_aggr_type,
        root_aggr_ptr,
        &mut Vec::new(),
        initializers,
        true,
    );

    // Always return true, even if `lower_to_stores` doesn't need to do any additional lowering,
    // because we already mem-cleared the aggregate.
    true
}

/// If `skip_zeros` is false, this is the default lowering, run if there are no any optimizations that we can perform.
/// It will flatten the aggregate structure and `store` initial values into individual fields.
/// Array fields might be an exception, depending on the size and the way the array is declared,
/// they might be lowered to `memcpy`s or even loops.
///
/// If `skip_zeros` is true, we are performing a lowering of only non-zero fields, knowing that the whole aggregate
/// is already initialized with `mem_clear_val`.
///
/// This function is called recursively for nested `init_aggr`s, starting from a `root_init_aggr`
/// whose aggregate pointer is `root_aggr_ptr`.
///
/// - `init_aggr`: The current `init_aggr` instruction to lower into the root aggregate at the position specified by `gep_indices`.
/// - `aggr_type`: The type of the aggregate initialized by the current `init_aggr`.
/// - `root_aggr_ptr`: The pointer to the root aggregate that is being initialized.
/// - `gep_indices`: The GEP indices to reach the position in the root aggregate where `init_aggr` initializes.
/// - `initializers`: The initializers to initialize the current `init_aggr`.
/// - `skip_zeros`: Whether to skip initializing zero values, because the root aggregate has already been initialized with `mem_clear_val`.
///
/// Returns `true` if the lowering was performed, `false` otherwise.
fn lower_to_stores<'a, 'b>(
    context: &'a mut Context<'b>,
    init_aggr: Value,
    aggr_type: Type,
    root_aggr_ptr: Value,
    gep_indices: &mut Vec<u64>,
    initializers: &[InitAggrInitializer],
    skip_zeroes: bool,
) -> bool {
    let init_aggr_metadata = init_aggr.get_metadata(context);
    match aggr_type.get_content(context).clone() {
        TypeContent::Array(arr_elem_type, length) => {
            assert_eq!(
                length as usize,
                initializers.len(),
                "`init_aggr` initializers must match the length of the array type"
            );

            // If all initializers are the same value, we can treat the array as a repeat array.
            // Returns the repeated element and the number of those elements in the array: `[repeated_value; size]`.
            // Single element arrays are treated as being non-repeat arrays. This is because
            // we can initialize them directly in-place without having a temporary `repeated_value`
            // that needs to be copied to every array element.
            fn as_repeat_array(
                initializers: &[InitAggrInitializer],
            ) -> Option<(InitAggrInitializer, u64)> {
                if initializers.len() == 1 {
                    return None;
                }

                initializers.split_first().and_then(|(first_init, rest)| {
                    if rest.iter().all(|init| init == first_init) {
                        Some((first_init.clone(), initializers.len() as u64))
                    } else {
                        None
                    }
                })
            }

            match as_repeat_array(initializers) {
                Some((initializer, length)) => {
                    let repeated_value = match initializer {
                        InitAggrInitializer::Value(value) => value,
                        InitAggrInitializer::NestedInitAggr {
                            load: nested_ia_load,
                            init_aggr: nested_init_aggr,
                        } => {
                            // The repeated initializer's value comes from an `init_aggr`.
                            // Note that we could store the entire nested aggregate into the first array element
                            // and then load it from there to initialize the rest of the array elements,
                            // thus eliminating the need for a temporary for the nested aggregate.
                            //
                            // But this actually harm optimization opportunities later on, unlike the case
                            // where we store the initializer into a temporary and then load it from there
                            // to initialize all array elements, which is what we do here.

                            let (nested_aggr_ptr, nested_ia_initializers) =
                                deconstruct_init_aggr(context, nested_init_aggr);

                            // Store the nested aggregate into its original temporary, **and not into the root aggregate**.
                            // Essentially, we are treating the nested `init_aggr` as a **root for the rest of the lowering**.
                            // Also, note that we are **not optimizing that new root for being an almost zero aggregate**.
                            // What we do, though, if we know that it is runtime-zeroed, we `mem_clear_val` it and pass
                            // the `skip_zeros = true` down its initialization chain.
                            let mut gep_indices: Vec<u64> = vec![];

                            let nested_aggr_type = nested_aggr_ptr
                                .match_ptr_type(context)
                                .expect("`nested_aggr_ptr` must be a pointer");

                            // **Note the we must proceed with lowering even if this initializer is
                            // runtime-zeroed and the root is already initialized with zeros.**
                            // The reason is the final removal of the lowered `init_aggr`s that happens
                            // below.
                            // Note that any instructions eventually inserted by the below
                            // `lower_to_stores` will in the end be DCEed because the temporary will
                            // not be used anywhere.

                            let is_temporary_runtime_zeroed = initializer.is_runtime_zeroed(context);

                            if is_temporary_runtime_zeroed {
                                // Insert `mem_clear_val` immediately after the `get_local` of the temporary.
                                InstructionInserter::after(context, nested_aggr_ptr)
                                    .mem_clear_val(nested_aggr_ptr);
                            }

                            lower_to_stores(
                                context,
                                nested_init_aggr,
                                nested_aggr_type,
                                nested_aggr_ptr,
                                &mut gep_indices,
                                &nested_ia_initializers,
                                // Pass `skip_zeros` to potential nested initializers
                                // if the temporary is runtime zeroed.
                                is_temporary_runtime_zeroed,
                            );

                            // Remove the `nested_init_aggr` and adapt its associated `load`
                            // to load from the `nested_aggr_ptr`.
                            // Note that we do not need to replace uses of the `nested_init_aggr`,
                            // because they are only used in their corresponding `load`,
                            // which we are adapting.
                            let nested_ia_block = nested_init_aggr
                                .get_parent_block(context)
                                .expect(
                                "`nested_init_aggr` is an instruction and must have a parent block",
                            );
                            nested_ia_block.remove_instruction(context, nested_init_aggr);
                            nested_ia_load.replace_instruction_value(
                                context,
                                nested_init_aggr,
                                nested_aggr_ptr,
                            );

                            // The value to use in the array initialization is the load from the nested aggregate.
                            nested_ia_load
                        }
                    };

                    // The repeat array initializer is runtime zeroed and the root aggregate is
                    // already zeroed. We can skip initializing this array.
                    // Note that this is the safe point to do it, because the eventual nested `init_aggr`s
                    // were removed above.
                    if initializer.is_runtime_zeroed(context) && skip_zeroes {
                        return true;
                    }

                    // For large repeating arrays, initialize them in a loop.
                    if length > 5 {
                        let array_ptr = if gep_indices.is_empty() {
                            // The array is the root aggregate, not nested in an other aggregate.
                            root_aggr_ptr
                        } else {
                            // The array is nested in an other aggregate. Calculate its pointer.
                            let inserter =
                                get_inst_inserter_for_before_init_aggr(context, init_aggr);
                            inserter
                                .get_elem_ptr_with_indices(root_aggr_ptr, aggr_type, gep_indices)
                                .add_metadatum(context, init_aggr_metadata)
                        };

                        generate_array_init_loop(
                            context,
                            array_ptr,
                            arr_elem_type,
                            repeated_value,
                            length,
                            init_aggr,
                            init_aggr_metadata,
                        );
                    } else {
                        // For small repeating arrays, store the `repeated_value` into each element individually.
                        for insert_idx in 0..length {
                            gep_indices.push(insert_idx);

                            let inserter =
                                get_inst_inserter_for_before_init_aggr(context, init_aggr);
                            let gep_val = inserter
                                .get_elem_ptr_with_indices(
                                    root_aggr_ptr,
                                    arr_elem_type,
                                    gep_indices,
                                )
                                .add_metadatum(context, init_aggr_metadata);

                            let inserter =
                                get_inst_inserter_for_before_init_aggr(context, init_aggr);
                            inserter
                                .store(gep_val, repeated_value)
                                .add_metadatum(context, init_aggr_metadata);

                            gep_indices.pop();
                        }
                    }
                }
                _ => {
                    // Non-repeating array initializers. Initialize each element individually.
                    for (insert_idx, initializer) in initializers.iter().enumerate() {
                        gep_indices.push(insert_idx as u64);

                        lower_single_initializer_to_stores(
                            context,
                            init_aggr,
                            root_aggr_ptr,
                            gep_indices,
                            init_aggr_metadata,
                            initializer,
                            arr_elem_type,
                            skip_zeroes,
                        );

                        gep_indices.pop();
                    }
                }
            }
        }
        TypeContent::Struct(field_types) => {
            assert_eq!(
                field_types.len(),
                initializers.len(),
                "`init_aggr` initializers must match the number of fields in the struct type"
            );
            for (insert_idx, (initializer, field_type)) in
                initializers.iter().zip(field_types).enumerate()
            {
                gep_indices.push(insert_idx as u64);

                lower_single_initializer_to_stores(
                    context,
                    init_aggr,
                    root_aggr_ptr,
                    gep_indices,
                    init_aggr_metadata,
                    initializer,
                    field_type,
                    skip_zeroes,
                );

                gep_indices.pop();
            }
        }
        _ => unreachable!("`aggr_ptr` must point to an array or struct IR type"),
    }

    true
}

fn get_inst_inserter_for_before_init_aggr<'a, 'b>(
    context: &'a mut Context<'b>,
    init_aggr: Value,
) -> InstructionInserter<'a, 'b> {
    InstructionInserter::before(context, init_aggr)
}

#[allow(clippy::too_many_arguments)]
fn lower_single_initializer_to_stores(
    context: &mut Context<'_>,
    init_aggr: Value,
    root_aggr_ptr: Value,
    gep_indices: &mut Vec<u64>,
    init_aggr_metadata: Option<MetadataIndex>,
    initializer: &InitAggrInitializer,
    elem_ty: Type,
    skip_zeroes: bool,
) {
    match initializer {
        InitAggrInitializer::Value(value) => {
            // This leaf value is runtime zeroed and the root aggregate is
            // already zeroed. We can skip initializing this leaf value.
            if initializer.is_runtime_zeroed(context) && skip_zeroes {
                return;
            }

            // The initializer's value does not come from a nested `init_aggr`.
            // Store the initializer's value directly into the field.
            let inserter = get_inst_inserter_for_before_init_aggr(context, init_aggr);
            let gep_val = inserter
                .get_elem_ptr_with_indices(root_aggr_ptr, elem_ty, gep_indices)
                .add_metadatum(context, init_aggr_metadata);

            let inserter = get_inst_inserter_for_before_init_aggr(context, init_aggr);
            inserter
                .store(gep_val, *value)
                .add_metadatum(context, init_aggr_metadata);
        }
        InitAggrInitializer::NestedInitAggr {
            load: nested_ia_load,
            init_aggr: nested_init_aggr,
        } => {
            // The initializer's value comes from an `init_aggr` which we want to lower
            // to stores into the root aggregate pointed by `root_aggr_ptr`.

            // We want to write nested `init_aggr`'s fields directly into the root aggregate's field.
            // This means completely removing the need for temporary storage of the nested aggregate,
            // and later `memcpy`ing it into the root aggregate.

            // **Note the we must proceed with lowering even if this initializer is
            // runtime-zeroed and the root is already initialized with zeros.**
            // The reason is the final removal of the lowered `init_aggr`s that happens
            // in this match arm, below.
            // Note that in that case, the `get_elem_ptr` instruction inserted below
            // will in the end be DCEed because it will not be used anywhere.

            let (nested_aggr_ptr, nested_ia_initializers) =
                deconstruct_init_aggr(context, *nested_init_aggr);

            let inserter = get_inst_inserter_for_before_init_aggr(context, *nested_init_aggr);
            let gep_val = inserter
                .get_elem_ptr_with_indices(root_aggr_ptr, elem_ty, gep_indices)
                .add_metadatum(context, init_aggr_metadata);

            let nested_aggr_type = nested_aggr_ptr
                .match_ptr_type(context)
                .expect("`nested_aggr_ptr` must be a pointer");

            lower_to_stores(
                context,
                *nested_init_aggr,
                nested_aggr_type,
                root_aggr_ptr,
                gep_indices,
                &nested_ia_initializers,
                skip_zeroes,
            );

            // Remove the `nested_init_aggr` and adapt its associated `load`
            // to load from the root aggregate's field pointer.
            // Note that we do not need to replace uses of the `nested_init_aggr`,
            // because they are only used in their corresponding `load`,
            // which we are adapting.
            let nested_ia_block = nested_init_aggr
                .get_parent_block(context)
                .expect("`nested_init_aggr` is an instruction and must have a parent block");
            nested_ia_block.remove_instruction(context, *nested_init_aggr);
            nested_ia_load.replace_instruction_value(context, *nested_init_aggr, gep_val);

            // The original local aggregate will after the lowering be unused and removed
            // later during DCE.
        }
    }
}

/// Find root `init_aggr` instructions in a `function`.
/// These are `init_aggr` instructions that are not nested in other `init_aggr` instructions.
///
/// Returns a vector of [Value]s representing the root `init_aggr` instructions, in post-order.
fn find_root_init_aggrs(context: &Context, function: Function) -> Vec<Value> {
    fn visit_nested_init_aggrs(
        context: &Context,
        parent_initializers: impl Iterator<Item = InitAggrInitializer>,
        nested_init_aggrs: &mut HashSet<Value>,
    ) {
        for initializer in parent_initializers {
            if let InitAggrInitializer::NestedInitAggr {
                load: _,
                init_aggr: init_aggr_val,
            } = initializer
            {
                let Some(Instruction {
                    parent: _,
                    op: InstOp::InitAggr(init_aggr),
                }) = init_aggr_val.get_instruction(context)
                else {
                    unreachable!("`init_aggr` is an `InstOp::InitAggr`");
                };
                nested_init_aggrs.insert(init_aggr_val);
                visit_nested_init_aggrs(
                    context,
                    init_aggr.initializers(context),
                    nested_init_aggrs,
                );
            }
        }
    }

    let mut result = vec![];
    let mut nested_init_aggrs = HashSet::new();

    // Traverse blocks in post-order and their instructions in reverse order.
    let po = dominator::compute_post_order(context, &function);
    for block in po.po_to_block.iter() {
        for inst in block.instruction_iter(context).rev() {
            if let Some(Instruction {
                parent: _,
                op: InstOp::InitAggr(init_aggr),
            }) = inst.get_instruction(context)
            {
                if !nested_init_aggrs.contains(&inst) {
                    // `inst` is a root `init_aggr`. Visit its nested `init_aggr`s.
                    result.push(inst);
                    visit_nested_init_aggrs(
                        context,
                        init_aggr.initializers(context),
                        &mut nested_init_aggrs,
                    );
                }
            }
        }
    }

    result
}

fn generate_array_init_loop(
    context: &mut Context,
    array_ptr: Value,
    elem_type: Type,
    repeated_value: Value,
    length: u64,
    init_aggr: Value,
    md_idx: Option<MetadataIndex>,
) {
    let block = init_aggr
        .get_parent_block(context)
        .expect("`init_aggr` is an instruction and must have a parent block");

    let init_aggr_idx = block
        .instruction_iter(context)
        .position(|v| v == init_aggr)
        .expect("`init_aggr` must be in its parent block");

    let (pre_block, exit_block) = block.split_at(context, init_aggr_idx + 1);

    exit_block.set_label(context, Some("array_init_loop_exit".into()));

    // Create the loop block before the exit block, with a single argument for the loop index.
    let loop_block = pre_block
        .get_function(context)
        .create_block_before(context, &exit_block, Some("array_init_loop".into()))
        .expect("`exit_block` exists in the `pre_block`'s function");
    let index_var_index = loop_block.new_arg(context, Type::get_uint64(context));
    let index = loop_block.get_arg(context, index_var_index).unwrap();

    // Start the loop by branching from the pre_block to the loop_block with index 0.
    let zero = Value::new_u64_constant(context, 0);
    pre_block.append(context).branch(loop_block, vec![zero]);

    // Build the loop block body.

    // 1. Store `repeated_value` into `array_ptr[index]`.
    let gep_val = loop_block
        .append(context)
        .get_elem_ptr(array_ptr, elem_type, vec![index]);
    loop_block
        .append(context)
        .store(gep_val, repeated_value)
        .add_metadatum(context, md_idx);

    // 2. Increment index by one.
    let one = Value::new_u64_constant(context, 1);
    let index_inc = loop_block
        .append(context)
        .binary_op(BinaryOpKind::Add, index, one);

    // 3. Compare index_inc with length to decide whether to continue the loop.
    //    continue = index_inc < length
    let len = Value::new_u64_constant(context, length);
    let r#continue = loop_block
        .append(context)
        .cmp(Predicate::LessThan, index_inc, len);

    // 4. If `continue` then `loop_block(index_inc)` else `exit_block()`.
    loop_block.append(context).conditional_branch(
        r#continue,
        loop_block,
        exit_block,
        vec![index_inc],
        vec![],
    );
}
