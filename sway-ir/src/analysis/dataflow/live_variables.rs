use crate::{
    AnalysisResult, AnalysisResults, BitField, Block, Context, DataFlowBlock, DataFlowDirection,
    Function, IrError, MeetOperator, Pass, PassMutability, ScopedPass,
};

use super::{solve_dataflow_equations, Gen, Kill, TransferFunction};

struct LiveVariableAnalysis;

pub fn all_variables() -> BitField {
    todo!()
}

impl Gen for LiveVariableAnalysis {
    fn gen(context: &Context, block: &Block) -> BitField {
        println!("i am here");
        println!("{block:?}, num-ins {:?}", block.num_instructions(context));
        let instructions = &context.blocks[block.0].instructions;
        for instruction_val in instructions {
            println!("{instruction_val:?}");
            let ins = instruction_val.get_instruction(context);
            println!("-- ins {ins:?}");
        }
        let variable_definitions = 
            instructions
            .iter()
            .map(|ins_value| { 
                println!("{ins_value:?}");
                ins_value.get_instruction(context)
            })
            .filter(|ins| {
                println!("-- {ins:?}");
                false
            });
            BitField::empty()
    }
}

impl Kill for LiveVariableAnalysis {
    fn kill(context: &Context, block: &Block) -> BitField {
        BitField::empty()
    }
}

pub fn compute_live_variable_pass(
    context: &Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<AnalysisResult, IrError> {
    Ok(Box::new(compute_live_variable(context, &function)))
}

pub fn compute_live_variable(context: &Context, function: &Function) -> Vec<DataFlowBlock> {
    let transfer_fn = TransferFunction {
        gen: LiveVariableAnalysis::gen,
        kill: LiveVariableAnalysis::kill,
    };
    let direction = DataFlowDirection::Backward;
    let meet_operator = MeetOperator::Union;
    solve_dataflow_equations(direction, meet_operator, *function, context, transfer_fn)
}

pub const LIVE_VARIABLE_NAME: &str = "livevar";

pub fn create_live_variable_pass() -> Pass {
    Pass {
        name: LIVE_VARIABLE_NAME,
        descr: "Live variable analysis",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Analysis(compute_live_variable_pass)),
    }
}
