//! ## Deduplicate functions.
//!
//! If two functions are functionally identical, eliminate one
//! and replace all calls to it with a call to the retained one.
//!
//! This pass shouldn't be required once the monomorphiser stops
//! generating a new function for each instantiation even when the exact
//! same instantiation exists.

use std::hash::{Hash, Hasher};

use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use crate::{
    build_call_graph, callee_first_order, function_print, AnalysisResults, Block, Context,
    DebugWithContext, Function, InstOp, Instruction, IrError, LocalVar, MetadataIndex, Metadatum,
    Module, Pass, PassMutability, ScopedPass, Type, Value,
};

pub const FN_DEDUP_DEBUG_PROFILE_NAME: &str = "fn-dedup-debug";
pub const FN_DEDUP_RELEASE_PROFILE_NAME: &str = "fn-dedup-release";
pub const FN_DEDUP_DEMONOMORPHIZE_NAME: &str = "fn-dedup-demonomorphize";

pub fn create_fn_dedup_release_profile_pass() -> Pass {
    Pass {
        name: FN_DEDUP_RELEASE_PROFILE_NAME,
        descr: "Function deduplication with metadata ignored",
        deps: vec![],
        runner: ScopedPass::ModulePass(PassMutability::Transform(dedup_fn_release_profile)),
    }
}

pub fn create_fn_dedup_debug_profile_pass() -> Pass {
    Pass {
        name: FN_DEDUP_DEBUG_PROFILE_NAME,
        descr: "Function deduplication with metadata considered",
        deps: vec![],
        runner: ScopedPass::ModulePass(PassMutability::Transform(dedup_fn_debug_profile)),
    }
}

pub fn create_fn_dedup_demonomorphize_pass() -> Pass {
    Pass {
        name: FN_DEDUP_DEMONOMORPHIZE_NAME,
        descr: "Function deduplication via demonomorphization",
        deps: vec![],
        runner: ScopedPass::ModulePass(PassMutability::Transform(dedup_fn_demonomorphize)),
    }
}

// Functions that are equivalent are put in the same set.
struct EqClass {
    // Map a function hash to its equivalence class.
    hash_set_map: FxHashMap<u64, FxHashSet<Function>>,
    // Once we compute the hash of a function, it's noted here.
    function_hash_map: FxHashMap<Function, u64>,
}

fn hash_fn(
    context: &Context,
    function: Function,
    eq_class: &mut EqClass,
    ignore_metadata: bool,
    ignore_pointee_type: bool,
) -> u64 {
    let state = &mut FxHasher::default();

    // A unique, but only in this function, ID for values.
    let localised_value_id: &mut FxHashMap<Value, u64> = &mut FxHashMap::default();
    // A unique, but only in this function, ID for blocks.
    let localised_block_id: &mut FxHashMap<Block, u64> = &mut FxHashMap::default();
    // A unique, but only in this function, ID for MetadataIndex.
    let metadata_hashes: &mut FxHashMap<MetadataIndex, u64> = &mut FxHashMap::default();
    // TODO: We could do a similar localised ID'ing of local variable names
    // and ASM block arguments too, thereby slightly relaxing the equality check.

    fn get_localised_id<T: Eq + Hash>(t: T, map: &mut FxHashMap<T, u64>) -> u64 {
        let cur_count = map.len();
        *map.entry(t).or_insert(cur_count as u64)
    }

    fn hash_value(
        context: &Context,
        v: Value,
        localised_value_id: &mut FxHashMap<Value, u64>,
        metadata_hashes: &mut FxHashMap<MetadataIndex, u64>,
        hasher: &mut FxHasher,
        ignore_metadata: bool,
    ) {
        let val = &context.values.get(v.0).unwrap().value;
        std::mem::discriminant(val).hash(hasher);
        match val {
            crate::ValueDatum::Argument(_) | crate::ValueDatum::Instruction(_) => {
                get_localised_id(v, localised_value_id).hash(hasher)
            }
            crate::ValueDatum::Constant(c) => c.hash(hasher),
        }
        if let Some(m) = &context.values.get(v.0).unwrap().metadata {
            if !ignore_metadata {
                hash_metadata(context, *m, metadata_hashes, hasher)
            }
        }
    }

    fn hash_type(context: &Context, hasher: &mut FxHasher, t: Type, ignore_pointee_type: bool) {
        if t.is_ptr(context) && ignore_pointee_type {
            std::mem::discriminant(t.get_content(context)).hash(hasher);
        } else {
            t.hash(hasher);
        }
    }

    fn hash_metadata(
        context: &Context,
        m: MetadataIndex,
        metadata_hashes: &mut FxHashMap<MetadataIndex, u64>,
        hasher: &mut FxHasher,
    ) {
        if let Some(hash) = metadata_hashes.get(&m) {
            return hash.hash(hasher);
        }

        let md_contents = context
            .metadata
            .get(m.0)
            .expect("Orphan / missing metadata");
        let descr = std::mem::discriminant(md_contents);
        let state = &mut FxHasher::default();
        // We temporarily set the discriminant as the hash.
        descr.hash(state);
        metadata_hashes.insert(m, state.finish());

        fn internal(
            context: &Context,
            m: &Metadatum,
            metadata_hashes: &mut FxHashMap<MetadataIndex, u64>,
            hasher: &mut FxHasher,
        ) {
            match m {
                Metadatum::Integer(i) => i.hash(hasher),
                Metadatum::Index(mdi) => hash_metadata(context, *mdi, metadata_hashes, hasher),
                Metadatum::String(s) => s.hash(hasher),
                Metadatum::SourceId(sid) => sid.hash(hasher),
                Metadatum::Struct(name, fields) => {
                    name.hash(hasher);
                    fields
                        .iter()
                        .for_each(|field| internal(context, field, metadata_hashes, hasher));
                }
                Metadatum::List(l) => l
                    .iter()
                    .for_each(|i| hash_metadata(context, *i, metadata_hashes, hasher)),
            }
        }
        internal(context, md_contents, metadata_hashes, hasher);

        let m_hash = state.finish();
        metadata_hashes.insert(m, m_hash);
        m_hash.hash(hasher);
    }

    // Start with the function return type.
    hash_type(
        context,
        state,
        function.get_return_type(context),
        ignore_pointee_type,
    );

    // ... and local variables.
    for (local_name, local_var) in function.locals_iter(context) {
        local_name.hash(state);
        if let Some(init) = local_var.get_initializer(context) {
            init.hash(state);
        }
        // Locals are pointers, so if we should ignore the pointee type, ignore the type of locals also.
        if !ignore_pointee_type {
            hash_type(
                context,
                state,
                local_var.get_type(context),
                ignore_pointee_type,
            );
        }
    }

    // Process every block, first its arguments and then the instructions.
    for block in function.block_iter(context) {
        get_localised_id(block, localised_block_id).hash(state);
        for &arg in block.arg_iter(context) {
            get_localised_id(arg, localised_value_id).hash(state);
            hash_type(
                context,
                state,
                arg.get_argument(context).unwrap().ty,
                ignore_pointee_type,
            );
        }
        for inst in block.instruction_iter(context) {
            get_localised_id(inst, localised_value_id).hash(state);
            let inst = inst.get_instruction(context).unwrap();
            std::mem::discriminant(&inst.op).hash(state);
            // Hash value inputs to instructions in one-go.
            for v in inst.op.get_operands() {
                hash_value(
                    context,
                    v,
                    localised_value_id,
                    metadata_hashes,
                    state,
                    ignore_metadata,
                );
            }
            // Hash non-value inputs.
            match &inst.op {
                crate::InstOp::AsmBlock(asm_block, args) => {
                    for arg in args
                        .iter()
                        .map(|arg| &arg.name)
                        .chain(asm_block.args_names.iter())
                    {
                        arg.as_str().hash(state);
                    }
                    if let Some(return_name) = &asm_block.return_name {
                        return_name.as_str().hash(state);
                    }
                    hash_type(context, state, asm_block.return_type, ignore_pointee_type);
                    for asm_inst in &asm_block.body {
                        asm_inst.op_name.as_str().hash(state);
                        for arg in &asm_inst.args {
                            arg.as_str().hash(state);
                        }
                        if let Some(imm) = &asm_inst.immediate {
                            imm.as_str().hash(state);
                        }
                    }
                }
                crate::InstOp::UnaryOp { op, .. } => op.hash(state),
                crate::InstOp::BinaryOp { op, .. } => op.hash(state),
                crate::InstOp::BitCast(_, ty) => {
                    hash_type(context, state, *ty, ignore_pointee_type)
                }
                crate::InstOp::Branch(b) => {
                    get_localised_id(b.block, localised_block_id).hash(state)
                }

                crate::InstOp::Call(callee, _) => {
                    match eq_class.function_hash_map.get(callee) {
                        Some(callee_hash) => {
                            callee_hash.hash(state);
                        }
                        None => {
                            // We haven't processed this callee yet. Just hash its name.
                            callee.get_name(context).hash(state);
                        }
                    }
                }
                crate::InstOp::CastPtr(_, ty) => {
                    hash_type(context, state, *ty, ignore_pointee_type)
                }
                crate::InstOp::Cmp(p, _, _) => p.hash(state),
                crate::InstOp::ConditionalBranch {
                    cond_value: _,
                    true_block,
                    false_block,
                } => {
                    get_localised_id(true_block.block, localised_block_id).hash(state);
                    get_localised_id(false_block.block, localised_block_id).hash(state);
                }
                crate::InstOp::ContractCall { name, .. } => {
                    name.hash(state);
                }
                crate::InstOp::FuelVm(fuel_vm_inst) => {
                    std::mem::discriminant(fuel_vm_inst).hash(state);
                    match fuel_vm_inst {
                        crate::FuelVmInstruction::Gtf { tx_field_id, .. } => {
                            tx_field_id.hash(state)
                        }
                        crate::FuelVmInstruction::Log { log_ty, .. } => {
                            hash_type(context, state, *log_ty, ignore_pointee_type)
                        }
                        crate::FuelVmInstruction::ReadRegister(reg) => reg.hash(state),
                        crate::FuelVmInstruction::Revert(_)
                        | crate::FuelVmInstruction::JmpMem
                        | crate::FuelVmInstruction::Smo { .. }
                        | crate::FuelVmInstruction::StateClear { .. }
                        | crate::FuelVmInstruction::StateLoadQuadWord { .. }
                        | crate::FuelVmInstruction::StateLoadWord(_)
                        | crate::FuelVmInstruction::StateStoreQuadWord { .. }
                        | crate::FuelVmInstruction::StateStoreWord { .. } => (),
                        crate::FuelVmInstruction::WideUnaryOp { op, .. } => op.hash(state),
                        crate::FuelVmInstruction::WideBinaryOp { op, .. } => op.hash(state),
                        crate::FuelVmInstruction::WideModularOp { op, .. } => op.hash(state),
                        crate::FuelVmInstruction::WideCmpOp { op, .. } => op.hash(state),
                        crate::FuelVmInstruction::Retd { ptr, len } => {
                            ptr.hash(state);
                            len.hash(state);
                        }
                    }
                }
                crate::InstOp::GetLocal(local) => function
                    .lookup_local_name(context, local)
                    .unwrap()
                    .hash(state),
                crate::InstOp::GetConfig(_, name) => name.hash(state),
                crate::InstOp::GetElemPtr { elem_ptr_ty, .. } => {
                    hash_type(context, state, *elem_ptr_ty, ignore_pointee_type)
                }
                crate::InstOp::IntToPtr(_, ty) => {
                    hash_type(context, state, *ty, ignore_pointee_type)
                }
                crate::InstOp::Load(_) => (),
                crate::InstOp::MemCopyBytes { byte_len, .. } => byte_len.hash(state),
                crate::InstOp::MemCopyVal { .. } | crate::InstOp::Nop => (),
                crate::InstOp::PtrToInt(_, ty) => {
                    hash_type(context, state, *ty, ignore_pointee_type)
                }
                crate::InstOp::Ret(_, ty) => hash_type(context, state, *ty, ignore_pointee_type),
                crate::InstOp::Store { .. } => (),
            }
        }
    }

    state.finish()
}

pub fn dedup_fns(
    context: &mut Context,
    _: &AnalysisResults,
    module: Module,
    ignore_metadata: bool,
) -> Result<bool, IrError> {
    let mut modified = false;
    let eq_class = &mut EqClass {
        hash_set_map: FxHashMap::default(),
        function_hash_map: FxHashMap::default(),
    };
    let cg = build_call_graph(context, &context.modules.get(module.0).unwrap().functions);
    let callee_first = callee_first_order(&cg);
    for function in callee_first {
        let hash = hash_fn(context, function, eq_class, ignore_metadata, false);
        eq_class
            .hash_set_map
            .entry(hash)
            .and_modify(|class| {
                class.insert(function);
            })
            .or_insert(vec![function].into_iter().collect());
        eq_class.function_hash_map.insert(function, hash);
    }

    // Let's go over the entire module, replacing calls to functions
    // with their representatives in the equivalence class.
    for function in module.function_iter(context) {
        let mut replacements = vec![];
        for (_block, inst) in function.instruction_iter(context) {
            let Some(Instruction {
                op: InstOp::Call(callee, args),
                ..
            }) = inst.get_instruction(context)
            else {
                continue;
            };
            let Some(callee_hash) = eq_class.function_hash_map.get(callee) else {
                continue;
            };
            // If the representative (first element in the set) is different, we need to replace.
            let Some(callee_rep) = eq_class
                .hash_set_map
                .get(callee_hash)
                .and_then(|f| f.iter().next())
                .filter(|rep| *rep != callee)
            else {
                continue;
            };
            replacements.push((inst, args.clone(), callee_rep));
        }
        if !replacements.is_empty() {
            modified = true;
        }
        for (inst, args, callee_rep) in replacements {
            inst.replace(
                context,
                crate::ValueDatum::Instruction(Instruction {
                    op: InstOp::Call(*callee_rep, args.clone()),
                    parent: inst.get_instruction(context).unwrap().parent,
                }),
            );
        }
    }

    Ok(modified)
}

fn dedup_fn_debug_profile(
    context: &mut Context,
    analysis_results: &AnalysisResults,
    module: Module,
) -> Result<bool, IrError> {
    dedup_fns(context, analysis_results, module, false)
}

fn dedup_fn_release_profile(
    context: &mut Context,
    analysis_results: &AnalysisResults,
    module: Module,
) -> Result<bool, IrError> {
    dedup_fns(context, analysis_results, module, true)
}

fn dedup_fn_demonomorphize(
    context: &mut Context,
    _analysis_results: &AnalysisResults,
    module: Module,
) -> Result<bool, IrError> {
    let modified = false;
    let eq_class = &mut EqClass {
        hash_set_map: FxHashMap::default(),
        function_hash_map: FxHashMap::default(),
    };
    let cg = build_call_graph(context, &context.modules.get(module.0).unwrap().functions);
    let callee_first = callee_first_order(&cg);
    for function in callee_first {
        let hash = hash_fn(context, function, eq_class, true, true);
        eq_class
            .hash_set_map
            .entry(hash)
            .and_modify(|class| {
                class.insert(function);
            })
            .or_insert(vec![function].into_iter().collect());
        eq_class.function_hash_map.insert(function, hash);
    }

    for (_class_id, class) in &eq_class.hash_set_map {
        if class.len() <= 1 {
            continue;
        }
        struct OthersTracker<'a> {
            locals_iter: Box<dyn Iterator<Item = (&'a String, &'a LocalVar)> + 'a>,
            instr_iter: Box<dyn Iterator<Item = (Block, Value)> + 'a>,
        }
        let mut class_iter = class.iter();
        let leader = class_iter.next().unwrap();
        let mut others: FxHashMap<_, _> = class_iter
            .map(|f| {
                (
                    *f,
                    OthersTracker {
                        locals_iter: Box::new(f.locals_iter(context)),
                        instr_iter: Box::new(f.instruction_iter(context)),
                    },
                )
            })
            .collect();

        // Collect those locals that need to be shifted to an argument.
        // The key is a local from the leader and the value is a list of
        // corresponding locals from others in the class.
        let mut locals_to_args = FxHashMap::default();
        for local in leader.locals_iter(context) {
            let mut other_locals = Vec::new();
            let mut shift_to_arg = false;
            // If this local differs from a corresponding one in others in the class,
            // we'll need to shift it to be a caller allocated parameter with an opaque
            // pointer passed as parameter.
            let ty = local.1.get_inner_type(context);
            for other_func in others.iter_mut() {
                let other_local = other_func.1.locals_iter.next().unwrap();
                assert!(
                    local.0 == other_local.0,
                    "If names differed, then the functions wouldn't be in the same class"
                );
                other_locals.push(other_local.1);
                let other_local_ty = other_local.1.get_inner_type(context);
                if ty != other_local_ty {
                    shift_to_arg = true;
                }
            }
            if shift_to_arg {
                locals_to_args.insert(local.1, other_locals);
            }
        }
        for (idx_in_block, (inst, block)) in leader
            .block_iter(context)
            .map(|b| {
                b.instruction_iter(context)
                    .map(move |inst| (inst, b))
                    .enumerate()
            })
            .flatten()
        {
            
        }
    }

    Ok(modified)
}
