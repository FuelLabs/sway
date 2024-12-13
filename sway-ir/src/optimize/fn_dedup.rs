//! ## Deduplicate functions.
//!
//! If two functions are functionally identical, eliminate one
//! and replace all calls to it with a call to the retained one.
//!
//! This pass shouldn't be required once the monomorphiser stops
//! generating a new function for each instantiation even when the exact
//! same instantiation exists.

use std::{
    hash::{Hash, Hasher},
    iter,
};

use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use crate::{
    build_call_graph, callee_first_order, AnalysisResults, Block, BlockArgument, Constant, Context,
    Function, InstOp, Instruction, InstructionInserter, IrError, LocalVar, MetadataIndex,
    Metadatum, Module, Pass, PassMutability, ScopedPass, Type, Value, ValueDatum,
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

    let mut dups_to_delete = vec![];

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
            dups_to_delete.push(*callee);
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

    // Replace config decode fns
    for config in module.iter_configs(context) {
        if let crate::ConfigContent::V1 { decode_fn, .. } = config {
            let f = decode_fn.get();

            let Some(callee_hash) = eq_class.function_hash_map.get(&f) else {
                continue;
            };

            // If the representative (first element in the set) is different, we need to replace.
            let Some(callee_rep) = eq_class
                .hash_set_map
                .get(callee_hash)
                .and_then(|f| f.iter().next())
                .filter(|rep| *rep != &f)
            else {
                continue;
            };

            dups_to_delete.push(decode_fn.get());
            decode_fn.replace(*callee_rep);
        }
    }

    // Remove replaced functions
    for function in dups_to_delete {
        module.remove_function(context, &function);
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
    // println!("{}", context);

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
            args_iter: Box<dyn Iterator<Item = (String, Value)> + 'a>,
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
                        args_iter: Box::new(f.args_iter(context).cloned()),
                    },
                )
            })
            .collect();

        // Note down arguments and retun value that need to be type erased.
        let mut type_erase_args = vec![];
        let mut type_erase_ret = false;
        for (arg_idx, arg) in leader.args_iter(context).enumerate() {
            let ty = arg.1.get_type(context).unwrap();
            if !ty.is_ptr(context) {
                continue;
            }
            for other_func in others.iter_mut() {
                let other_arg = other_func.1.args_iter.next().unwrap();
                let other_arg_ty = other_arg.1.get_type(context).unwrap();
                assert!(
                    other_arg_ty.is_ptr(context),
                    "Functions wouldn't be in the same class if args differ"
                );
                if ty.get_pointee_type(context).unwrap()
                    != other_arg_ty.get_pointee_type(context).unwrap()
                {
                    type_erase_args.push(arg_idx);
                    break;
                }
            }
        }

        let ret_ty = leader.get_return_type(context);
        let mut ret_ty_map = FxHashMap::default();
        ret_ty_map.insert(*leader, ret_ty);
        if ret_ty.is_ptr(context) {
            for other_func in others.iter_mut() {
                let other_ret_ty = other_func.0.get_return_type(context);
                ret_ty_map.insert(*other_func.0, other_ret_ty);
                assert!(
                    other_ret_ty.is_ptr(context),
                    "Function't wouldn't be in the same class if ret type differs"
                );
                if ret_ty.get_pointee_type(context).unwrap()
                    != other_ret_ty.get_pointee_type(context).unwrap()
                {
                    type_erase_ret = true;
                }
            }
        }

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
                other_locals.push(*other_local.1);
                let other_local_ty = other_local.1.get_inner_type(context);
                if ty != other_local_ty {
                    shift_to_arg = true;
                }
            }
            if shift_to_arg {
                locals_to_args.insert(*local.1, other_locals);
            }
        }

        let mut can_optimize = true;

        #[derive(Default)]
        struct ChangeInstrs {
            // all the CastPtr/IntToPtr that need change in the leader
            cast_to_ptr: FxHashSet<Value>,
            // all the GetLocal that need change in the leader
            get_local: FxHashSet<Value>,
            // All the GEPs Map<in the leader, in others> that need to become a
            // "add pointer + offset", where offset if parameterized instruction.
            gep: FxHashMap<Value, Vec<Value>>,
            // All the MemCopyVals Map<in the leader, in others> that need to become
            // MemCopyBytes with the size parameterized.
            mem_copy_val: FxHashMap<Value, Vec<Value>>,
        }

        let mut type_erase_block_args = FxHashSet::default();
        let mut change_instrs = ChangeInstrs::default();
        'leader_loop: for (block, inst) in leader.instruction_iter(context) {
            let mut block_args_checked = false;
            for other_func in others.iter_mut() {
                let (other_block, other_instr) = other_func.1.instr_iter.next().unwrap();
                // Check if any of the block args (except for the entry block) need their type erased.
                if !block_args_checked && leader.get_entry_block(context) != block {
                    block_args_checked = true;
                    for (arg_idx, arg) in block.arg_iter(context).enumerate() {
                        let ty = arg.get_type(context).unwrap();
                        if !ty.is_ptr(context) {
                            continue;
                        }
                        let other_ty = other_block
                            .get_arg(context, arg_idx)
                            .unwrap()
                            .get_type(context)
                            .unwrap();
                        assert!(
                            other_ty.is_ptr(context),
                            "If this isn't a pointer, functions shouldn't be in same class"
                        );
                        if ty.get_pointee_type(context).unwrap()
                            != other_ty.get_pointee_type(context).unwrap()
                        {
                            type_erase_block_args.insert(*arg);
                        }
                    }
                }
                // Throughout this loop we check only for differing types between the leader and
                // its followers. Other differences aren't checked for because then the hashes would
                // be different and they wouldn't be in the same class.
                match &inst.get_instruction(context).unwrap().op {
                    InstOp::AsmBlock(asm_block, _args) => {
                        let InstOp::AsmBlock(other_asm_block, _) =
                            &other_instr.get_instruction(context).unwrap().op
                        else {
                            panic!("Leader and follower are different instructions in same class");
                        };
                        if asm_block.return_type != other_asm_block.return_type {
                            can_optimize = false;
                            break 'leader_loop;
                        }
                    }
                    InstOp::UnaryOp { .. } => {}
                    InstOp::BinaryOp { .. } => {}
                    InstOp::BitCast(_value, ty) => {
                        let InstOp::BitCast(_other_value, other_ty) =
                            &other_instr.get_instruction(context).unwrap().op
                        else {
                            panic!("Leader and follower are different instructions in same class");
                        };
                        if ty != other_ty {
                            can_optimize = false;
                            break 'leader_loop;
                        }
                    }
                    InstOp::Branch(..) => {}
                    InstOp::Call(..) => {}
                    InstOp::CastPtr(_, target_ty) | InstOp::IntToPtr(_, target_ty) => {
                        match &other_instr.get_instruction(context).unwrap().op {
                            InstOp::CastPtr(_, other_target_ty)
                            | InstOp::IntToPtr(_, other_target_ty) => {
                                if target_ty != other_target_ty {
                                    change_instrs.cast_to_ptr.insert(inst);
                                }
                            }
                            _ => {
                                panic!(
                                    "Leader and follower are different instructions in same class"
                                );
                            }
                        }
                    }
                    InstOp::Cmp(..) => {}
                    InstOp::ConditionalBranch { .. } => {}
                    InstOp::ContractCall { .. } => {}
                    InstOp::FuelVm(_fuel_vm_instruction) => {}
                    InstOp::GetLocal(local_var) => {
                        if locals_to_args.contains_key(local_var) {
                            change_instrs.get_local.insert(inst);
                        }
                    }
                    InstOp::GetConfig(..) => {}
                    InstOp::GetElemPtr {
                        elem_ptr_ty: _,
                        indices,
                        base,
                    } => {
                        let InstOp::GetElemPtr {
                            elem_ptr_ty: _,
                            indices: other_indices,
                            base: other_base,
                        } = &other_instr.get_instruction(context).unwrap().op
                        else {
                            panic!("Leader and follower are different instructions in same class");
                        };
                        let base_ty = base
                            .get_type(context)
                            .unwrap()
                            .get_pointee_type(context)
                            .unwrap();
                        let other_base_ty = other_base
                            .get_type(context)
                            .unwrap()
                            .get_pointee_type(context)
                            .unwrap();
                        if base_ty != other_base_ty {
                            // If we can't determine the offset to a compile time constant,
                            // we cannot do the optimization.
                            if base_ty.get_value_indexed_offset(context, indices).is_none()
                                || other_base_ty
                                    .get_value_indexed_offset(context, &other_indices)
                                    .is_none()
                            {
                                can_optimize = false;
                                break 'leader_loop;
                            }
                            change_instrs
                                .gep
                                .entry(inst)
                                .and_modify(|others| others.push(other_instr))
                                .or_insert(vec![other_instr]);
                        }
                    }
                    InstOp::Load(_value) => {}
                    InstOp::MemCopyBytes { .. } => {}
                    InstOp::MemCopyVal { dst_val_ptr, .. } => {
                        let InstOp::MemCopyVal {
                            dst_val_ptr: other_dst_val_ptr,
                            ..
                        } = &other_instr.get_instruction(context).unwrap().op
                        else {
                            panic!("Leader and follower are different instructions in same class");
                        };
                        let copied_ty = dst_val_ptr.get_type(context).unwrap();
                        let other_copied_ty = other_dst_val_ptr.get_type(context).unwrap();
                        if copied_ty != other_copied_ty {
                            change_instrs
                                .mem_copy_val
                                .entry(inst)
                                .and_modify(|others| others.push(other_instr))
                                .or_insert(vec![other_instr]);
                        }
                    }
                    InstOp::Nop => {}
                    InstOp::PtrToInt(..) => {}
                    InstOp::Ret(..) => {}
                    InstOp::Store { .. } => {}
                }
            }
        }

        if !can_optimize {
            continue;
        }

        if change_instrs.cast_to_ptr.is_empty()
            && change_instrs.gep.is_empty()
            && change_instrs.get_local.is_empty()
            && change_instrs.mem_copy_val.is_empty()
        {
            continue;
        }

        // Map every function in the class to an index. Useful later on.
        let class_fn_to_idx: FxHashMap<_, _> = iter::once(leader)
            .chain(others.keys())
            .enumerate()
            .map(|(idx, f)| (*f, idx))
            .collect();

        // Note down all call sites for later use.
        let call_sites = context
            .module_iter()
            .flat_map(|module| module.function_iter(context))
            .flat_map(|ref call_from_func| {
                call_from_func
                    .block_iter(context)
                    .flat_map(|ref block| {
                        block
                            .instruction_iter(context)
                            .filter_map(|instr_val| {
                                if let Instruction {
                                    op: InstOp::Call(call_to_func, _),
                                    ..
                                } = instr_val
                                    .get_instruction(context)
                                    .expect("`instruction_iter()` must return instruction values.")
                                {
                                    iter::once(leader)
                                        .chain(others.keys())
                                        .contains(call_to_func)
                                        .then_some((*call_from_func, *block, instr_val))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // `others` captures `context`, so let's drop it now.
        drop(others);

        let unit_ptr_ty = Type::new_ptr(context, Type::get_unit(context));

        // Track the additional arguments we're adding.
        #[derive(Clone)]
        enum NewArg {
            // A local which is now allocated in the caller and
            // whose pointer is passed as parameter.
            CallerAllocatedLocal(LocalVar),
            // A u64 value
            Size(u64),
        }
        // New arguments for the leader followed by every other.
        let mut new_args: Vec<Vec<NewArg>> = vec![Vec::new(); class.len()];
        // Argument number for a local
        let mut local_to_argno = FxHashMap::default();

        // We'll collect all the new arguments first,
        // and then actually add them and modify the instructions.
        for (local, other_locals) in locals_to_args {
            new_args[0].push(NewArg::CallerAllocatedLocal(local));
            for (i, ty) in other_locals
                .iter()
                .map(|other_local| NewArg::CallerAllocatedLocal(*other_local))
                .enumerate()
            {
                new_args[i + 1].push(ty);
            }
            local_to_argno.insert(local, new_args[0].len() - 1);
        }

        // Map a GEP or MemCopyVal to the new size parameter.
        let mut gep_memcpyval_to_argno = FxHashMap::default();

        for (inst, other_insts) in change_instrs
            .gep
            .iter()
            .chain(change_instrs.mem_copy_val.iter())
        {
            let mut this_params: Vec<u64> = Vec::new();
            for inst in std::iter::once(inst).chain(other_insts) {
                match &inst.get_instruction(context).unwrap().op {
                    InstOp::GetElemPtr {
                        elem_ptr_ty: _,
                        indices,
                        base,
                    } => {
                        let base_ty = base
                            .get_type(context)
                            .unwrap()
                            .get_pointee_type(context)
                            .unwrap();
                        let offset = base_ty.get_value_indexed_offset(context, indices).unwrap();
                        this_params.push(offset);
                    }
                    InstOp::MemCopyVal {
                        dst_val_ptr,
                        src_val_ptr: _,
                    } => {
                        let copied_ty = dst_val_ptr
                            .get_type(context)
                            .unwrap()
                            .get_pointee_type(context)
                            .unwrap();
                        let size_copied_type_bytes = copied_ty.size(context).in_bytes();
                        this_params.push(size_copied_type_bytes);
                    }
                    _ => {
                        unreachable!("Expected only GEPs or MemCopyVals")
                    }
                }
            }
            assert!(this_params.len() == class.len());
            // Check if any row in new_args is already the same as this_params,
            // in which case we can reuse that parameter.
            let argno = (0..new_args[0].len()).find_map(|i| {
                if matches!(new_args[0][i], NewArg::CallerAllocatedLocal(..)) {
                    return None;
                }
                let ith_params: Vec<_> = new_args
                    .iter()
                    .map(|params| {
                        let NewArg::Size(size_param) = params[i] else {
                            panic!("We just filtered for Size parameters above");
                        };
                        size_param
                    })
                    .collect();
                (this_params == ith_params).then_some(i)
            });
            if let Some(argno) = argno {
                gep_memcpyval_to_argno.insert(inst, argno);
            } else {
                let argno = new_args[0].len();
                gep_memcpyval_to_argno.insert(inst, argno);

                // Let's add a new row to new_args.
                for (i, param) in this_params.iter().enumerate() {
                    new_args[i].push(NewArg::Size(*param));
                }
            }
        }

        // We are now equipped to actually modify the program.
        // 1(a) Type erase existing arguments / return type if necessary
        for arg_idx in &type_erase_args {
            let arg_val = leader.get_ith_arg(context, *arg_idx);
            let arg = arg_val.get_argument_mut(context).unwrap();
            arg.ty = unit_ptr_ty;
        }
        for block_arg in type_erase_block_args {
            let arg = block_arg.get_argument_mut(context).unwrap();
            arg.ty = unit_ptr_ty;
        }
        if type_erase_ret {
            leader.set_return_type(context, unit_ptr_ty);
        }

        // 1(b) Add the new arguments.
        let mut new_arg_values = Vec::with_capacity(new_args[0].len());
        let entry_block = leader.get_entry_block(context);
        for (arg_idx, new_arg) in (&new_args[0]).iter().enumerate() {
            let (new_block_arg, new_arg_name) = match new_arg {
                NewArg::CallerAllocatedLocal(..) => (
                    BlockArgument {
                        block: entry_block,
                        idx: leader.num_args(context),
                        ty: unit_ptr_ty,
                    },
                    "demonomorphize_alloca_arg_".to_string() + &arg_idx.to_string(),
                ),
                NewArg::Size(_) => (
                    BlockArgument {
                        block: entry_block,
                        idx: leader.num_args(context),
                        ty: Type::get_uint64(context),
                    },
                    "demonomorphize_size_arg_".to_string() + &arg_idx.to_string(),
                ),
            };
            let new_arg_value = Value::new_argument(context, new_block_arg);
            leader.add_arg(context, new_arg_name, new_arg_value);
            entry_block.add_arg(context, new_arg_value);
            new_arg_values.push(new_arg_value);
        }

        // 2. Modify pointer casts.
        for cast_to_ptr in change_instrs.cast_to_ptr {
            let instr = cast_to_ptr.get_instruction(context).unwrap();
            let new_instr = match &instr.op {
                InstOp::CastPtr(source, _target_ty) => InstOp::CastPtr(*source, unit_ptr_ty),
                InstOp::IntToPtr(source, _target_ty) => InstOp::IntToPtr(*source, unit_ptr_ty),
                _ => unreachable!(),
            };
            let new_instr = ValueDatum::Instruction(Instruction {
                op: new_instr,
                parent: instr.parent,
            });
            cast_to_ptr.replace(context, new_instr);
        }

        // 3. Modify GEPs.
        for (gep, _) in &change_instrs.gep {
            let instr = gep.get_instruction(context).unwrap();
            let InstOp::GetElemPtr {
                elem_ptr_ty,
                indices: _,
                base,
            } = instr.op
            else {
                panic!("Should be GEP");
            };
            let arg_idx = gep_memcpyval_to_argno[gep];
            let arg_value = new_arg_values[arg_idx];
            let parent_block = instr.parent;

            let replacement_add = Value::new_instruction(
                context,
                parent_block,
                InstOp::BinaryOp {
                    op: crate::BinaryOpKind::Add,
                    arg1: base,
                    arg2: arg_value,
                },
            );
            let mut inserter = InstructionInserter::new(
                context,
                parent_block,
                crate::InsertionPosition::Before(*gep),
            );
            inserter.insert(replacement_add);
            let ptr_cast = ValueDatum::Instruction(Instruction {
                parent: parent_block,
                op: InstOp::CastPtr(replacement_add, elem_ptr_ty),
            });

            gep.replace(context, ptr_cast);
        }

        // 4. Modify MemCopyVals
        for (mem_copy_val, _) in &change_instrs.mem_copy_val {
            let instr = mem_copy_val.get_instruction(context).unwrap();
            let InstOp::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } = &instr.op
            else {
                panic!("Should be MemCopyVal");
            };
            let arg_idx = gep_memcpyval_to_argno[mem_copy_val];
            let arg_value = new_arg_values[arg_idx];
            let replacement_memcpybyte = ValueDatum::Instruction(Instruction {
                op: InstOp::MemCopyBytes {
                    dst_val_ptr: *dst_val_ptr,
                    src_val_ptr: *src_val_ptr,
                    byte_len: arg_value,
                },
                parent: instr.parent,
            });
            mem_copy_val.replace(context, replacement_memcpybyte);
        }

        // 5. Update the uses of get_local instructions to directly use the argument.
        let mut replacements = FxHashMap::default();
        for get_local in &change_instrs.get_local {
            let InstOp::GetLocal(local_var) = get_local.get_instruction(context).unwrap().op else {
                panic!("Expected GetLocal");
            };
            let arg = local_to_argno.get(&local_var).unwrap();
            replacements.insert(*get_local, new_arg_values[*arg]);
        }
        leader.replace_values(context, &replacements, None);

        // 6. Finally modify calls to each function in the class.
        for (caller, call_block, call_inst) in call_sites {
            // Update the callee in call_inst first, all calls go to the leader now.
            let (callee, params) = {
                let InstOp::Call(callee, params) =
                    &mut call_inst.get_instruction_mut(context).unwrap().op
                else {
                    panic!("Expected Call");
                };
                let original_callee = *callee;
                *callee = *leader;
                (original_callee, params)
            };

            // Update existing params to erase type, if necessary.
            let mut new_params = params.clone();
            let mut new_instrs = vec![];
            for arg_idx in &type_erase_args {
                let new_param = Value::new_instruction(
                    context,
                    call_block,
                    InstOp::CastPtr(new_params[*arg_idx], unit_ptr_ty),
                );
                new_instrs.push(new_param);
                new_params[*arg_idx] = new_param;
            }
            let mut inserter = InstructionInserter::new(
                context,
                call_block,
                crate::InsertionPosition::Before(call_inst),
            );
            inserter.insert_slice(&new_instrs);
            let InstOp::Call(_callee, params) =
                &mut call_inst.get_instruction_mut(context).unwrap().op
            else {
                panic!("Expected Call");
            };
            *params = new_params;

            // Now add the new args.
            let callee_idx = class_fn_to_idx[&callee];
            let new_args = &new_args[callee_idx];
            for new_arg in new_args {
                match new_arg {
                    NewArg::CallerAllocatedLocal(original_local) => {
                        let name = callee
                            .lookup_local_name(context, original_local)
                            .cloned()
                            .unwrap_or("".to_string())
                            + "_demonomorphized";
                        let new_local = caller.new_unique_local_var(
                            context,
                            name,
                            original_local
                                .get_type(context)
                                .get_pointee_type(context)
                                .unwrap(),
                            original_local.get_initializer(context).cloned(),
                            original_local.is_mutable(context),
                        );
                        let new_local_ptr = Value::new_instruction(
                            context,
                            call_block,
                            InstOp::GetLocal(new_local),
                        );
                        let new_local_ptr_casted = Value::new_instruction(
                            context,
                            call_block,
                            InstOp::CastPtr(new_local_ptr, unit_ptr_ty),
                        );
                        let mut inserter = InstructionInserter::new(
                            context,
                            call_block,
                            crate::InsertionPosition::Before(call_inst),
                        );
                        inserter.insert_slice(&[new_local_ptr, new_local_ptr_casted]);
                        let InstOp::Call(_, args) =
                            &mut call_inst.get_instruction_mut(context).unwrap().op
                        else {
                            panic!("Expected Call");
                        };
                        args.push(new_local_ptr_casted);
                    }
                    NewArg::Size(val) => {
                        let new_size_const = Constant::new_uint(context, 64, *val);
                        let new_size_arg = Value::new_constant(context, new_size_const);
                        let InstOp::Call(_, args) =
                            &mut call_inst.get_instruction_mut(context).unwrap().op
                        else {
                            panic!("Expected Call");
                        };
                        args.push(new_size_arg);
                    }
                }
            }
            if type_erase_ret {
                let inserter = InstructionInserter::new(
                    context,
                    call_block,
                    crate::InsertionPosition::After(call_inst),
                );
                let ret_cast = inserter.cast_ptr(call_inst, ret_ty_map[&callee]);
                caller.replace_value(context, call_inst, ret_cast, None);
                // caller.replace_value will replace call_inst in the just inserted cast. Fix it.
                let Instruction {
                    op: InstOp::CastPtr(ptr, _),
                    ..
                } = ret_cast.get_instruction_mut(context).unwrap()
                else {
                    panic!("We just created this to be a Castptr");
                };
                *ptr = call_inst;

                // Modify all return instructions
                for (_, ret) in leader
                    .instruction_iter(context)
                    .filter(|inst| {
                        matches!(inst.1.get_instruction(context).unwrap().op, InstOp::Ret(..))
                    })
                    .collect::<Vec<_>>()
                {
                    let InstOp::Ret(__entry, ty) =
                        &mut ret.get_instruction_mut(context).unwrap().op
                    else {
                        panic!("We just filtered for Rets")
                    };
                    *ty = unit_ptr_ty;
                }
            }
        }
    }

    Ok(modified)
}
