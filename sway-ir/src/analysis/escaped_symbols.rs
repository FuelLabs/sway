//! An analysis to compute symbols that escape out from a function.
//! This could be into another function, or via ptr_to_int etc.
//! Any transformations involving such symbols are unsafe.
use rustc_hash::FxHashSet;

use crate::{
    AnalysisResult, AnalysisResultT, AnalysisResults, BlockArgument, Context, Function,
    Instruction, IrError, LocalVar, Pass, PassMutability, ScopedPass, Type, Value, ValueDatum,
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

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum Symbol {
    Local(LocalVar),
    Arg(BlockArgument),
}

impl Symbol {
    pub fn get_type(&self, context: &Context) -> Type {
        match self {
            Symbol::Local(l) => l.get_type(context),
            Symbol::Arg(ba) => ba.ty,
        }
    }

    pub fn _get_name(&self, context: &Context, function: Function) -> String {
        match self {
            Symbol::Local(l) => function.lookup_local_name(context, l).unwrap().clone(),
            Symbol::Arg(ba) => format!("{}[{}]", ba.block.get_label(context), ba.idx),
        }
    }
}

// A value may (indirectly) refer to one or more symbols.
pub fn get_symbols(context: &Context, val: Value) -> Vec<Symbol> {
    let mut visited = FxHashSet::default();
    fn get_symbols_rec(
        context: &Context,
        visited: &mut FxHashSet<Value>,
        val: Value,
    ) -> Vec<Symbol> {
        if visited.contains(&val) {
            return vec![];
        }
        visited.insert(val);
        match context.values[val.0].value {
            ValueDatum::Instruction(Instruction::GetLocal(local)) => vec![Symbol::Local(local)],
            ValueDatum::Instruction(Instruction::GetElemPtr { base, .. }) => {
                get_symbols_rec(context, visited, base)
            }
            ValueDatum::Argument(b) => {
                if b.block.get_label(context) == "entry" {
                    vec![Symbol::Arg(b)]
                } else {
                    b.block
                        .pred_iter(context)
                        .map(|pred| b.get_val_coming_from(context, pred).unwrap())
                        .flat_map(|v| get_symbols_rec(context, visited, v))
                        .collect()
                }
            }
            _ => vec![],
        }
    }
    get_symbols_rec(context, &mut visited, val)
}

pub fn get_symbol(context: &Context, val: Value) -> Option<Symbol> {
    let syms = get_symbols(context, val);
    (syms.len() == 1).then(|| syms[0])
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
