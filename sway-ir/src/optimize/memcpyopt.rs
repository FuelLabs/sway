//! Optimisations related to mem_copy.
//! - replace a `store` directly from a `load` with a `mem_copy_val`.

use crate::{
    AnalysisResults, Context, Function, Instruction, IrError, Pass, PassMutability, ScopedPass,
    TypeContent, Value,
};

pub const MEMCPYOPT_NAME: &str = "memcpyopt";

pub fn create_memcpyopt_pass() -> Pass {
    Pass {
        name: MEMCPYOPT_NAME,
        descr: "Memcopy optimization.",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(mem_copy_opt)),
    }
}

pub fn mem_copy_opt(
    context: &mut Context,
    _analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    // Currently we have only one optimisation:
    load_store_to_memcopy(context, function)
}

fn load_store_to_memcopy(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find any `store`s of `load`s.  These can be replaced with `mem_copy` and are especially
    // important for non-copy types on architectures which don't support loading them.
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            instr_val
                .get_instruction(context)
                .and_then(|instr| {
                    // Is the instruction a Store?
                    if let Instruction::Store {
                        dst_val_ptr,
                        stored_val,
                    } = instr
                    {
                        stored_val
                            .get_instruction(context)
                            .map(|src_instr| (src_instr, dst_val_ptr))
                    } else {
                        None
                    }
                })
                .and_then(|(src_instr, dst_val_ptr)| {
                    // Is the Store source a Load?
                    if let Instruction::Load(src_val_ptr) = src_instr {
                        Some((block, instr_val, *dst_val_ptr, *src_val_ptr))
                    } else {
                        None
                    }
                })
                .and_then(|candidate @ (_block, _store_val, dst_ptr, _src_ptr)| {
                    // XXX TEMPORARY 'FIX':
                    //
                    // We need to do proper aliasing analysis for this pass.  It's possible to have
                    // the following:
                    //
                    // X = load ptr A       -- dereference A
                    // store Y to ptr A     -- mutate A
                    // store X to ptr B     -- store original A to B
                    //
                    // Which this pass would convert to:
                    //
                    //                      -- DCE the load
                    // store Y to ptr A     -- mutate A
                    // memcpy ptr B, ptr A  -- copy _mutated_ A to B
                    //
                    // To temporarily avoid this problem we're not going to mem_copy copy types and
                    // assume (oh, no) that larger types subject to this pass aren't mutated.  This
                    // only works for now because it has always worked in the past, but there are
                    // no guarantees this couldn't flare up somewhere.
                    dst_ptr
                        .get_type(context)
                        .and_then(|ptr_ty| ptr_ty.get_inner_type(context))
                        .map(|ty| match ty.get_content(context) {
                            TypeContent::Unit | TypeContent::Bool | TypeContent::Pointer(_) => {
                                false
                            }
                            TypeContent::Uint(bits) => *bits > 64,
                            _ => true,
                        })?
                        .then_some(candidate)
                })
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    for (block, store_val, dst_val_ptr, src_val_ptr) in candidates {
        let mem_copy_val = Value::new_instruction(
            context,
            Instruction::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            },
        );
        block.replace_instruction(context, store_val, mem_copy_val)?;
    }

    Ok(true)
}
