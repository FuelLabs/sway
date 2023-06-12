//! An analysis to compute symbols that escape out from a function.
//! This could be into another function, or via ptr_to_int etc.
//! Any transformations involving such symbols are unsafe.
use rustc_hash::FxHashSet;

use crate::{
    get_symbols, AnalysisResult, AnalysisResultT, AnalysisResults, Context, Function, Instruction,
    IrError, Pass, PassMutability, ScopedPass, Symbol, Value,
};

pub const ESCAPED_SYMBOLS_NAME: &str = "escaped_symbols";

pub fn create_escaped_symbols_pass() -> Pass {
    Pass {
        name: ESCAPED_SYMBOLS_NAME,
        descr: "Symbols that escape / cannot be analysed",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Analysis(compute_escaped_symbols_pass)),
    }
}

pub type EscapedSymbols = FxHashSet<Symbol>;
impl AnalysisResultT for EscapedSymbols {}

pub fn compute_escaped_symbols_pass(
    context: &Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<AnalysisResult, IrError> {
    Ok(Box::new(compute_escaped_symbols(context, &function)))
}

pub fn compute_escaped_symbols(context: &Context, function: &Function) -> EscapedSymbols {
    let mut result = FxHashSet::default();

    let add_from_val = |result: &mut FxHashSet<Symbol>, val: &Value| {
        get_symbols(context, *val).iter().for_each(|s| {
            result.insert(*s);
        });
    };

    for (_block, inst) in function.instruction_iter(context) {
        match inst.get_instruction(context).unwrap() {
            Instruction::AsmBlock(_, args) => {
                for arg_init in args.iter().filter_map(|arg| arg.initializer) {
                    add_from_val(&mut result, &arg_init)
                }
            }
            Instruction::BinaryOp { .. } => (),
            Instruction::BitCast(_, _) => (),
            Instruction::Branch(_) => (),
            Instruction::Call(_, args) => args.iter().for_each(|v| add_from_val(&mut result, v)),
            Instruction::CastPtr(_, _) => (),
            Instruction::Cmp(_, _, _) => (),
            Instruction::ConditionalBranch { .. } => (),
            Instruction::ContractCall { params, .. } => add_from_val(&mut result, params),
            Instruction::FuelVm(_) => (),
            Instruction::GetLocal(_) => (),
            Instruction::GetElemPtr { .. } => (),
            Instruction::IntToPtr(_, _) => (),
            Instruction::Load(_) => (),
            Instruction::MemCopyBytes { .. } => (),
            Instruction::MemCopyVal { .. } => (),
            Instruction::Nop => (),
            Instruction::PtrToInt(v, _) => add_from_val(&mut result, v),
            Instruction::Ret(_, _) => (),
            Instruction::Store { .. } => (),
        }
    }

    result
}
