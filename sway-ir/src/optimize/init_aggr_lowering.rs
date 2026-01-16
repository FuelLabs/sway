//! Lowering of the `init_aggr` instruction.
//!
//! The lowering replaces `init_aggr` with an optimal sequence of
//! instructions like `store`, `mem_clear_val`, `mem_copy_val`, etc.

use std::{collections::HashSet, vec};

use rustc_hash::FxHashMap;

use crate::{
    AnalysisResults, BinaryOpKind, Context, Function, InitAggrInitializer, InsertionPosition, InstOp, Instruction, InstructionInserter, IrError, MetadataIndex, Pass, PassMutability, Predicate, ScopedPass, Type, TypeContent, Value, dominator::{self}
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
    // Lowering does not remove the `root_init_aggr` instruction yet, nor replaces its uses.
    // This is done after all lowerings are complete, and for that we build the `replace_map`
    // which maps each `root_init_aggr` to the aggregate that it initializes.
    let mut replace_map = FxHashMap::<Value, Value>::default();
    for root_init_aggr in root_init_aggrs.iter() {
        let (root_aggr_ptr, initializers) = deconstruct_init_aggr(context, *root_init_aggr);

        replace_map.insert(*root_init_aggr, root_aggr_ptr);

        let aggr_type = root_aggr_ptr
            .match_ptr_type(context)
            .expect("`root_aggr_ptr` must be a pointer");

        // TODO: (INIT-AGGR) Think of other possible optimizations that bring benefits, if any.
        // Try mostly optimized lowerings first.
        let _ = lower_mostly_zeroed_aggregate()
            || lower_to_stores(
                context,
                *root_init_aggr,
                aggr_type,
                root_aggr_ptr,
                &mut Vec::new(),
                &initializers,
            );
    }

    // Replace all usages of `root_init_aggr`s with the pointers to the aggregates they initialize.
    function.replace_values(context, &replace_map, None);

    // Finally, remove all root `root_init_aggr` instructions.
    function.remove_instructions(context, |inst| root_init_aggrs.contains(&inst));

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

/// This lowering checks whether the aggregate being initialized is mostly zeroed,
/// i.e., whether most of its fields are initialized to zero values.
/// If so, it lowers the `init_aggr` to a `mem_clear_val` for the entire aggregate,
/// followed by `store`s for the non-zero fields.
///
/// E.g., a very common case is initializing tuples like `(0, 0, 0, some_variable)`.
///
/// Returns `true` if the lowering was performed, `false` otherwise.
fn lower_mostly_zeroed_aggregate() -> bool {
    // TODO: (INIT-AGGR) Implement lowering of mostly zeroed aggregates.
    false
}

/// This is the default lowering, run if there are no any optimizations that we can perform.
/// It will flatten the aggregate structure and `store` initial values into individual fields.
/// Array fields might be an exception, depending on the size and the way the array is declared,
/// they might be lowered to `memcpy`s or even loops.
///
/// This function is called recursively for nested `init_aggr`s, starting from a `root_init_aggr`
/// whose aggregate pointer is `root_aggr_ptr`.
///
/// - `init_aggr`: The current `init_aggr` instruction to lower into the root aggregate at the position specified by `gep_indices`.
/// - `aggr_type`: The type of the aggregate initialized by the current `init_aggr`.
/// - `root_aggr_ptr`: The pointer to the root aggregate that is being initialized.
/// - `gep_indices`: The GEP indices to reach the position in the root aggregate where `init_aggr` initializes.
///
/// Returns `true` if the lowering was performed, `false` otherwise.
fn lower_to_stores<'a, 'b>(
    context: &'a mut Context<'b>,
    init_aggr: Value,
    aggr_type: Type,
    root_aggr_ptr: Value,
    gep_indices: &mut Vec<u64>,
    initializers: &[InitAggrInitializer],
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
            fn as_repeat_array(
                initializers: &[InitAggrInitializer],
            ) -> Option<(InitAggrInitializer, u64)> {
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

                            // Store the nested aggregate into its original temporary, and not into the root aggregate.
                            // Essentially, we are treating the nested `init_aggr` as a root for the rest of the lowering.
                            let mut gep_indices: Vec<u64> = vec![];

                            let nested_aggr_type = nested_aggr_ptr
                                .match_ptr_type(context)
                                .expect("`nested_aggr_ptr` must be a pointer");

                            lower_to_stores(
                                context,
                                nested_init_aggr,
                                nested_aggr_type,
                                nested_aggr_ptr,
                                &mut gep_indices,
                                &nested_ia_initializers,
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
                                .get_elem_ptr_with_idcs(root_aggr_ptr, aggr_type, gep_indices)
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
                                .get_elem_ptr_with_idcs(root_aggr_ptr, arr_elem_type, gep_indices)
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
                None => {
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
    let block = init_aggr
        .get_parent_block(context)
        .expect("`init_aggr` is an instruction and must have a parent block");
    InstructionInserter::new(context, block, InsertionPosition::Before(init_aggr))
}

fn lower_single_initializer_to_stores(
    context: &mut Context<'_>,
    init_aggr: Value,
    root_aggr_ptr: Value,
    gep_indices: &mut Vec<u64>,
    init_aggr_metadata: Option<MetadataIndex>,
    initializer: &InitAggrInitializer,
    elem_ty: Type,
) {
    match initializer {
        InitAggrInitializer::Value(value) => {
            // The initializer's value does not come from a nested `init_aggr`.
            // Store the initializer's value directly into the field.
            let inserter = get_inst_inserter_for_before_init_aggr(context, init_aggr);
            let gep_val = inserter
                .get_elem_ptr_with_idcs(root_aggr_ptr, elem_ty, gep_indices)
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

            let (nested_aggr_ptr, nested_ia_initializers) =
                deconstruct_init_aggr(context, *nested_init_aggr);

            let inserter = get_inst_inserter_for_before_init_aggr(context, *nested_init_aggr);
            let gep_val = inserter
                .get_elem_ptr_with_idcs(root_aggr_ptr, elem_ty, gep_indices)
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
