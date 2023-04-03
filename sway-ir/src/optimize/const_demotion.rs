///! Constant value demotion.
///!
///! This pass demotes 'by-value' constant types to 'by-reference` pointer types, based on target
///! specific parameters.
///!
///! Storage for constant values is created on the stack in variables which are initialized with the
///! original values.
use crate::{
    AnalysisResults, Block, Constant, Context, Function, IrError, Pass, PassMutability, ScopedPass,
    Value,
};

use rustc_hash::FxHashMap;

pub const CONSTDEMOTION_NAME: &str = "constdemotion";

pub fn create_const_demotion_pass() -> Pass {
    Pass {
        name: CONSTDEMOTION_NAME,
        descr: "By-value constant demotion to by-reference.",
        deps: Vec::new(),
        runner: ScopedPass::FunctionPass(PassMutability::Transform(const_demotion)),
    }
}

pub fn const_demotion(
    context: &mut Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    // Find all candidate constant values and their wrapped constants.
    let candidate_values = function
        .instruction_iter(context)
        .flat_map(|(_block, inst)| inst.get_instruction(context).unwrap().get_operands())
        .filter_map(|val| {
            val.get_constant(context).and_then(|c| {
                super::target_fuel::is_demotable_type(context, &c.ty).then(|| (val, c.clone()))
            })
        })
        .collect::<Vec<_>>();

    if candidate_values.is_empty() {
        return Ok(false);
    }

    // Create a new entry block to initialise and load the constants.
    let (const_init_block, orig_entry_block) =
        function.get_entry_block(context).split_at(context, 0);

    // Insert const initialisation into new init block, gather into a replacement map.
    let replace_map =
        FxHashMap::from_iter(candidate_values.into_iter().map(|(old_value, constant)| {
            (
                old_value,
                demote(context, &function, &const_init_block, &constant),
            )
        }));

    // Terminate the init block.
    const_init_block
        .ins(context)
        .branch(orig_entry_block, Vec::new());

    // Replace the value.
    function.replace_values(context, &replace_map, Some(orig_entry_block));

    assert_eq!(const_init_block, function.get_entry_block(context));

    Ok(true)
}

fn demote(context: &mut Context, function: &Function, block: &Block, constant: &Constant) -> Value {
    // Create a variable for const.
    let var = function.new_unique_local_var(
        context,
        "__const".to_owned(),
        constant.ty,
        Some(constant.clone()),
    );

    // Create local_var and load instructions.
    let var_val = block.ins(context).get_local(var);
    block.ins(context).load(var_val)
}
