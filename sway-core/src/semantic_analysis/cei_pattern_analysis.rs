// CEI stands for "Checks, Effects, Interactions".  We check that no storage writes
// (effects) occur after calling external contracts (interaction) and issue warnings
// if it's the case.
// See this [blog post](https://fravoll.github.io/solidity-patterns/checks_effects_interactions.html)
// for more detail on vulnerabilities in case of storage modification after interaction
// and this [blog post](https://chainsecurity.com/curve-lp-oracle-manipulation-post-mortem)
// for more information on storage reads after interaction.
// We also treat the balance tree reads and writes separately,
// as well as modifying output messages.

use crate::{
    decl_engine::*,
    language::{
        ty::{self, TyFunctionDecl, TyImplSelfOrTrait},
        AsmOp,
    },
    Engines,
};
use std::fmt;
use std::{collections::HashSet, sync::Arc};
use sway_error::warning::{CompileWarning, Warning};
use sway_types::{Ident, Span, Spanned};

#[derive(PartialEq, Eq, Hash, Clone)]
enum Effect {
    Interaction,  // interaction with external contracts
    StorageWrite, // storage modification
    StorageRead,  // storage read
    // Note: there are no operations that only write to the balance tree
    BalanceTreeRead,      // balance tree read operation
    BalanceTreeReadWrite, // balance tree read and write operation
    OutputMessage,        // operation creates a new `Output::Message`
    MintAsset,            // mint operation
    BurnAsset,            // burn operation
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Effect::*;
        match self {
            Interaction => write!(f, "Interaction"),
            StorageWrite => write!(f, "Storage write"),
            StorageRead => write!(f, "Storage read"),
            BalanceTreeRead => write!(f, "Balance tree read"),
            BalanceTreeReadWrite => write!(f, "Balance tree update"),
            OutputMessage => write!(f, "Output message sent"),
            MintAsset => write!(f, "Asset minted"),
            BurnAsset => write!(f, "Asset burned"),
        }
    }
}

impl Effect {
    fn to_suggestion(&self) -> String {
        use Effect::*;
        String::from(match self {
            Interaction => "making all interactions",
            StorageWrite => "making all storage writes",
            StorageRead => "making all storage reads",
            BalanceTreeRead => "making all balance tree reads",
            BalanceTreeReadWrite => "making all balance tree updates",
            OutputMessage => "sending all output messages",
            MintAsset => "minting assets",
            BurnAsset => "burning assets",
        })
    }
}

// The algorithm that searches for storage operations after interaction
// is organized as an automaton.
// After an interaction is found in a code block, we keep looking either for
// storage reads or writes.
enum CEIAnalysisState {
    LookingForInteraction, // initial state of the automaton
    LookingForEffect,
}

pub(crate) fn analyze_program(engines: &Engines, prog: &ty::TyProgram) -> Vec<CompileWarning> {
    match &prog.kind {
        // Libraries, scripts, or predicates can't access storage
        // so we don't analyze these
        ty::TyProgramKind::Library { .. }
        | ty::TyProgramKind::Script { .. }
        | ty::TyProgramKind::Predicate { .. } => vec![],
        ty::TyProgramKind::Contract { .. } => {
            analyze_contract(engines, &prog.root_module.all_nodes)
        }
    }
}

fn analyze_contract(engines: &Engines, ast_nodes: &[ty::TyAstNode]) -> Vec<CompileWarning> {
    let decl_engine = engines.de();
    let mut warnings: Vec<CompileWarning> = vec![];
    for fn_decl in contract_entry_points(decl_engine, ast_nodes) {
        // no need to analyze the entry fn
        if fn_decl.name.as_str() == "__entry" {
            continue;
        }
        analyze_code_block(engines, &fn_decl.body, &fn_decl.name, &mut warnings);
    }
    warnings
}

// standalone functions and methods
fn contract_entry_points(
    decl_engine: &DeclEngine,
    ast_nodes: &[ty::TyAstNode],
) -> Vec<Arc<ty::TyFunctionDecl>> {
    use crate::ty::TyAstNodeContent::Declaration;
    ast_nodes
        .iter()
        .flat_map(|ast_node| match &ast_node.content {
            Declaration(ty::TyDecl::FunctionDecl(ty::FunctionDecl { decl_id, .. })) => {
                decl_id_to_fn_decls(decl_engine, decl_id)
            }
            Declaration(ty::TyDecl::ImplSelfOrTrait(ty::ImplSelfOrTrait { decl_id, .. })) => {
                impl_trait_methods(decl_engine, decl_id)
            }
            _ => vec![],
        })
        .collect()
}

fn decl_id_to_fn_decls(
    decl_engine: &DeclEngine,
    decl_id: &DeclId<TyFunctionDecl>,
) -> Vec<Arc<TyFunctionDecl>> {
    vec![decl_engine.get_function(decl_id)]
}

fn impl_trait_methods(
    decl_engine: &DeclEngine,
    impl_trait_decl_id: &DeclId<TyImplSelfOrTrait>,
) -> Vec<Arc<ty::TyFunctionDecl>> {
    let impl_trait = decl_engine.get_impl_self_or_trait(impl_trait_decl_id);
    impl_trait
        .items
        .iter()
        .flat_map(|item| match item {
            ty::TyImplItem::Fn(fn_decl) => Some(fn_decl),
            ty::TyImplItem::Constant(_) => None,
            ty::TyImplItem::Type(_) => None,
        })
        .flat_map(|fn_decl| decl_id_to_fn_decls(decl_engine, &fn_decl.id().clone()))
        .collect()
}

// This is the main part of the analysis algorithm:
// we are looking for various effects after contract interaction
fn analyze_code_block(
    engines: &Engines,
    codeblock: &ty::TyCodeBlock,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    let mut interaction_span: Span = Span::dummy();
    let mut codeblock_effects = HashSet::new();
    let mut analysis_state: CEIAnalysisState = CEIAnalysisState::LookingForInteraction;

    for ast_node in &codeblock.contents {
        let codeblock_entry_effects =
            analyze_code_block_entry(engines, ast_node, block_name, warnings);
        match analysis_state {
            CEIAnalysisState::LookingForInteraction => {
                if codeblock_entry_effects.contains(&Effect::Interaction) {
                    analysis_state = CEIAnalysisState::LookingForEffect;
                    interaction_span = ast_node.span.clone();
                }
            }
            CEIAnalysisState::LookingForEffect => warn_after_interaction(
                &codeblock_entry_effects,
                &interaction_span,
                &ast_node.span,
                block_name,
                warnings,
            ),
        };
        codeblock_effects.extend(codeblock_entry_effects)
    }
    codeblock_effects
}

fn analyze_code_block_entry(
    engines: &Engines,
    entry: &ty::TyAstNode,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    match &entry.content {
        ty::TyAstNodeContent::Declaration(decl) => {
            analyze_codeblock_decl(engines, decl, block_name, warnings)
        }
        ty::TyAstNodeContent::Expression(expr) => {
            analyze_expression(engines, expr, block_name, warnings)
        }
        ty::TyAstNodeContent::SideEffect(_) | ty::TyAstNodeContent::Error(_, _) => HashSet::new(),
    }
}

fn analyze_codeblock_decl(
    engines: &Engines,
    decl: &ty::TyDecl,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    // Declarations (except variable declarations) are not allowed in a codeblock
    use crate::ty::TyDecl::*;
    match decl {
        VariableDecl(var_decl) => analyze_expression(engines, &var_decl.body, block_name, warnings),
        _ => HashSet::new(),
    }
}

fn analyze_expression(
    engines: &Engines,
    expr: &ty::TyExpression,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    use crate::ty::TyExpressionVariant::*;
    let decl_engine = engines.de();
    match &expr.expression {
        // base cases: no warnings can be emitted
        Literal(_)
        | ConstantExpression { .. }
        | ConfigurableExpression { .. }
        | ConstGenericExpression { .. }
        | VariableExpression { .. }
        | FunctionParameter
        | StorageAccess(_)
        | Break
        | Continue
        | AbiName(_) => effects_of_expression(engines, expr),
        Reassignment(reassgn) => analyze_expression(engines, &reassgn.rhs, block_name, warnings),
        CodeBlock(codeblock) => analyze_code_block(engines, codeblock, block_name, warnings),
        LazyOperator {
            lhs: left,
            rhs: right,
            ..
        }
        | ArrayIndex {
            prefix: left,
            index: right,
        } => analyze_two_expressions(engines, left, right, block_name, warnings),
        FunctionApplication {
            arguments,
            fn_ref,
            selector,
            call_path,
            ..
        } => {
            let func = decl_engine.get_function(fn_ref);
            // we don't need to run full analysis on the function body as it will be covered
            // as a separate step of the whole contract analysis
            // we just need function's effects at this point
            let fn_effs = effects_of_codeblock(engines, &func.body);

            // assuming left-to-right arguments evaluation
            // we run CEI violation analysis as if the arguments form a code block
            let args_effs = analyze_expressions(
                engines,
                arguments.iter().map(|arg| &arg.expr),
                block_name,
                warnings,
            );
            if args_effs.contains(&Effect::Interaction) {
                // TODO: interaction span has to be more precise and point to an argument which performs interaction
                let last_arg_span = &arguments.last().unwrap().expr.span;
                warn_after_interaction(
                    &fn_effs,
                    &call_path.span(),
                    last_arg_span,
                    block_name,
                    warnings,
                )
            }

            let mut result_effs = set_union(fn_effs, args_effs);
            if selector.is_some() {
                // external contract call (a.k.a. interaction)
                result_effs.extend(HashSet::from([Effect::Interaction]))
            };
            result_effs
        }
        IntrinsicFunction(intrinsic) => {
            let intr_effs = effects_of_intrinsic(&intrinsic.kind);
            // assuming left-to-right arguments evaluation
            let args_effs =
                analyze_expressions(engines, intrinsic.arguments.iter(), block_name, warnings);
            if args_effs.contains(&Effect::Interaction) {
                // TODO: interaction span has to be more precise and point to an argument which performs interaction
                warn_after_interaction(&intr_effs, &expr.span, &expr.span, block_name, warnings)
            }
            set_union(intr_effs, args_effs)
        }
        Tuple { fields: exprs }
        | ArrayExplicit {
            elem_type: _,
            contents: exprs,
        } => {
            // assuming left-to-right fields/elements evaluation
            analyze_expressions(engines, exprs.iter(), block_name, warnings)
        }
        ArrayRepeat {
            elem_type: _,
            value,
            length,
        } => {
            // assuming left-to-right fields/elements evaluation
            let mut cond_then_effs = analyze_expression(engines, value, block_name, warnings);
            cond_then_effs.extend(analyze_expression(engines, length, block_name, warnings));
            cond_then_effs
        }
        StructExpression { fields, .. } => {
            // assuming left-to-right fields evaluation
            analyze_expressions(
                engines,
                fields.iter().map(|e| &e.value),
                block_name,
                warnings,
            )
        }
        StructFieldAccess { prefix: expr, .. }
        | TupleElemAccess { prefix: expr, .. }
        | ImplicitReturn(expr)
        | Return(expr)
        | Panic(expr)
        | EnumTag { exp: expr }
        | UnsafeDowncast { exp: expr, .. }
        | AbiCast { address: expr, .. }
        | Ref(expr)
        | Deref(expr) => analyze_expression(engines, expr, block_name, warnings),
        EnumInstantiation { contents, .. } => match contents {
            Some(expr) => analyze_expression(engines, expr, block_name, warnings),
            None => HashSet::new(),
        },
        MatchExp { desugared, .. } => analyze_expression(engines, desugared, block_name, warnings),
        IfExp {
            condition,
            then,
            r#else,
        } => {
            let cond_then_effs =
                analyze_two_expressions(engines, condition, then, block_name, warnings);
            let cond_else_effs = match r#else {
                Some(else_exp) => {
                    analyze_two_expressions(engines, condition, else_exp, block_name, warnings)
                }
                None => HashSet::new(),
            };
            set_union(cond_then_effs, cond_else_effs)
        }
        WhileLoop { condition, body } => {
            // if the loop (condition + body) contains both interaction and state effects
            // in _any_ order, we report CEI pattern violation
            let cond_effs = analyze_expression(engines, condition, block_name, warnings);
            let body_effs = analyze_code_block(engines, body, block_name, warnings);
            let res_effs = set_union(cond_effs, body_effs);
            if res_effs.contains(&Effect::Interaction) {
                // TODO: the span is not very precise, we can do better here, but this
                // will need a bit of refactoring of the CEI analysis
                let span = expr.span.clone();
                warn_after_interaction(&res_effs, &span, &span, &block_name.clone(), warnings)
            }
            res_effs
        }
        ForLoop { desugared } => analyze_expression(engines, desugared, block_name, warnings),
        AsmExpression {
            registers, body, ..
        } => {
            let init_exprs = registers
                .iter()
                .filter_map(|rdecl| rdecl.initializer.as_ref());
            let init_effs = analyze_expressions(engines, init_exprs, block_name, warnings);
            let asmblock_effs = analyze_asm_block(body, block_name, warnings);
            if init_effs.contains(&Effect::Interaction) {
                // TODO: improve locations accuracy
                warn_after_interaction(&asmblock_effs, &expr.span, &expr.span, block_name, warnings)
            }
            set_union(init_effs, asmblock_effs)
        }
    }
}

fn analyze_two_expressions(
    engines: &Engines,
    first: &ty::TyExpression,
    second: &ty::TyExpression,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    let first_effs = analyze_expression(engines, first, block_name, warnings);
    let second_effs = analyze_expression(engines, second, block_name, warnings);
    if first_effs.contains(&Effect::Interaction) {
        warn_after_interaction(
            &second_effs,
            &first.span,
            &second.span,
            block_name,
            warnings,
        )
    }
    set_union(first_effs, second_effs)
}

// Analyze a sequence of expressions
// TODO: analyze_expressions, analyze_codeblock and analyze_asm_block (see below) are very similar in structure
//       looks like the algorithm implementation should be generalized
fn analyze_expressions<'a>(
    engines: &Engines,
    expressions: impl Iterator<Item = &'a ty::TyExpression>,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    let mut interaction_span: Span = Span::dummy();
    let mut accumulated_effects = HashSet::new();
    let mut analysis_state: CEIAnalysisState = CEIAnalysisState::LookingForInteraction;

    for expr in expressions {
        let expr_effs = analyze_expression(engines, expr, block_name, warnings);
        match analysis_state {
            CEIAnalysisState::LookingForInteraction => {
                if expr_effs.contains(&Effect::Interaction) {
                    analysis_state = CEIAnalysisState::LookingForEffect;
                    interaction_span = expr.span.clone();
                }
            }
            CEIAnalysisState::LookingForEffect => warn_after_interaction(
                &expr_effs,
                &interaction_span,
                &expr.span,
                block_name,
                warnings,
            ),
        };
        accumulated_effects.extend(expr_effs)
    }
    accumulated_effects
}

// No need to worry about jumps because they are not allowed in `asm` blocks.
fn analyze_asm_block(
    asm_block: &Vec<AsmOp>,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    let mut interaction_span: Span = Span::dummy();
    let mut accumulated_effects = HashSet::new();
    let mut analysis_state: CEIAnalysisState = CEIAnalysisState::LookingForInteraction;

    for asm_op in asm_block {
        let asm_op_effs = effects_of_asm_op(asm_op);
        match analysis_state {
            CEIAnalysisState::LookingForInteraction => {
                if asm_op_effs.contains(&Effect::Interaction) {
                    analysis_state = CEIAnalysisState::LookingForEffect;
                    interaction_span = asm_op.span.clone();
                }
            }
            CEIAnalysisState::LookingForEffect => warn_after_interaction(
                &asm_op_effs,
                &interaction_span,
                &asm_op.span,
                block_name,
                warnings,
            ),
        };
        accumulated_effects.extend(asm_op_effs)
    }
    accumulated_effects
}

fn warn_after_interaction(
    ast_node_effects: &HashSet<Effect>,
    interaction_span: &Span,
    effect_span: &Span,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) {
    let interaction_singleton = HashSet::from([Effect::Interaction]);
    let state_effects = ast_node_effects.difference(&interaction_singleton);
    for eff in state_effects {
        warnings.push(CompileWarning {
            span: Span::join(interaction_span.clone(), effect_span),
            warning_content: Warning::EffectAfterInteraction {
                effect: eff.to_string(),
                effect_in_suggestion: Effect::to_suggestion(eff),
                block_name: block_name.clone(),
            },
        });
    }
}

fn effects_of_codeblock_entry(engines: &Engines, ast_node: &ty::TyAstNode) -> HashSet<Effect> {
    match &ast_node.content {
        ty::TyAstNodeContent::Declaration(decl) => effects_of_codeblock_decl(engines, decl),
        ty::TyAstNodeContent::Expression(expr) => effects_of_expression(engines, expr),
        ty::TyAstNodeContent::SideEffect(_) | ty::TyAstNodeContent::Error(_, _) => HashSet::new(),
    }
}

fn effects_of_codeblock_decl(engines: &Engines, decl: &ty::TyDecl) -> HashSet<Effect> {
    use crate::ty::TyDecl::*;
    match decl {
        VariableDecl(var_decl) => effects_of_expression(engines, &var_decl.body),
        // Declarations (except variable declarations) are not allowed in the body of a function
        _ => HashSet::new(),
    }
}

fn effects_of_expression(engines: &Engines, expr: &ty::TyExpression) -> HashSet<Effect> {
    use crate::ty::TyExpressionVariant::*;
    let type_engine = engines.te();
    let decl_engine = engines.de();
    match &expr.expression {
        Literal(_)
        | ConstantExpression { .. }
        | ConfigurableExpression { .. }
        | VariableExpression { .. }
        | FunctionParameter
        | Break
        | Continue
        | ConstGenericExpression { .. }
        | AbiName(_) => HashSet::new(),
        // this type of assignment only mutates local variables and not storage
        Reassignment(reassgn) => effects_of_expression(engines, &reassgn.rhs),
        StorageAccess(_) => match &*type_engine.get(expr.return_type) {
            // accessing a storage map's method (or a storage vector's method),
            // which is represented using a struct with empty fields
            // does not result in a storage read
            crate::TypeInfo::Struct(decl_ref)
                if decl_engine.get_struct(decl_ref).fields.is_empty() =>
            {
                HashSet::new()
            }
            // if it's an empty enum then it cannot be constructed and hence cannot be read
            // adding this check here just to be on the safe side
            crate::TypeInfo::Enum(decl_ref)
                if decl_engine.get_enum(decl_ref).variants.is_empty() =>
            {
                HashSet::new()
            }
            _ => HashSet::from([Effect::StorageRead]),
        },
        LazyOperator { lhs, rhs, .. }
        | ArrayIndex {
            prefix: lhs,
            index: rhs,
        } => {
            let mut effs = effects_of_expression(engines, lhs);
            let rhs_effs = effects_of_expression(engines, rhs);
            effs.extend(rhs_effs);
            effs
        }
        Tuple { fields: exprs }
        | ArrayExplicit {
            elem_type: _,
            contents: exprs,
        } => effects_of_expressions(engines, exprs),
        ArrayRepeat {
            elem_type: _,
            value,
            length,
        } => {
            let mut effs = effects_of_expression(engines, value);
            effs.extend(effects_of_expression(engines, length));
            effs
        }
        StructExpression { fields, .. } => effects_of_struct_expressions(engines, fields),
        CodeBlock(codeblock) => effects_of_codeblock(engines, codeblock),
        MatchExp { desugared, .. } => effects_of_expression(engines, desugared),
        IfExp {
            condition,
            then,
            r#else,
        } => {
            let mut effs = effects_of_expression(engines, condition);
            effs.extend(effects_of_expression(engines, then));
            let else_effs = match r#else {
                Some(expr) => effects_of_expression(engines, expr),
                None => HashSet::new(),
            };
            effs.extend(else_effs);
            effs
        }
        StructFieldAccess { prefix: expr, .. }
        | TupleElemAccess { prefix: expr, .. }
        | EnumTag { exp: expr }
        | UnsafeDowncast { exp: expr, .. }
        | ImplicitReturn(expr)
        | Return(expr)
        | Panic(expr)
        | Ref(expr)
        | Deref(expr) => effects_of_expression(engines, expr),
        EnumInstantiation { contents, .. } => match contents {
            Some(expr) => effects_of_expression(engines, expr),
            None => HashSet::new(),
        },
        AbiCast { address, .. } => effects_of_expression(engines, address),
        IntrinsicFunction(intr_fn) => effects_of_expressions(engines, &intr_fn.arguments)
            .union(&effects_of_intrinsic(&intr_fn.kind))
            .cloned()
            .collect(),
        WhileLoop { condition, body } => effects_of_expression(engines, condition)
            .union(&effects_of_codeblock(engines, body))
            .cloned()
            .collect(),
        ForLoop { desugared } => effects_of_expression(engines, desugared),
        FunctionApplication {
            fn_ref,
            arguments,
            selector,
            ..
        } => {
            let fn_body = &decl_engine.get_function(fn_ref).body;
            let mut effs = effects_of_codeblock(engines, fn_body);
            let args_effs =
                map_hashsets_union(arguments, |e| effects_of_expression(engines, &e.expr));
            effs.extend(args_effs);
            if selector.is_some() {
                // external contract call (a.k.a. interaction)
                effs.extend(HashSet::from([Effect::Interaction]))
            };
            effs
        }
        AsmExpression {
            registers,
            body,
            whole_block_span: _,
            ..
        } => effects_of_register_initializers(engines, registers)
            .union(&effects_of_asm_ops(body))
            .cloned()
            .collect(),
    }
}

fn effects_of_intrinsic(intr: &sway_ast::Intrinsic) -> HashSet<Effect> {
    use sway_ast::Intrinsic::*;
    match intr {
        StateClear | StateStoreWord | StateStoreQuad => HashSet::from([Effect::StorageWrite]),
        StateLoadWord | StateLoadQuad => HashSet::from([Effect::StorageRead]),
        Smo => HashSet::from([Effect::OutputMessage]),
        ContractCall => HashSet::from([Effect::Interaction]),
        Revert
        | JmpMem
        | IsReferenceType
        | IsStrArray
        | SizeOfType
        | SizeOfVal
        | SizeOfStr
        | ContractRet
        | AssertIsStrArray
        | ToStrArray
        | Eq
        | Gt
        | Lt
        | Gtf
        | AddrOf
        | Log
        | Add
        | Sub
        | Mul
        | Div
        | And
        | Or
        | Xor
        | Mod
        | Rsh
        | Lsh
        | PtrAdd
        | PtrSub
        | Not
        | EncodeBufferEmpty
        | EncodeBufferAppend
        | EncodeBufferAsRawSlice
        | Slice
        | ElemAt
        | Transmute
        | Dbg
        | Alloc
        | RuntimeMemoryId
        | EncodingMemoryId => HashSet::new(),
    }
}

fn effects_of_asm_op(op: &AsmOp) -> HashSet<Effect> {
    match op.op_name.as_str().to_lowercase().as_str() {
        "scwq" | "sww" | "swwq" => HashSet::from([Effect::StorageWrite]),
        "srw" | "srwq" => HashSet::from([Effect::StorageRead]),
        "tr" | "tro" => HashSet::from([Effect::BalanceTreeReadWrite]),
        "bal" => HashSet::from([Effect::BalanceTreeRead]),
        "smo" => HashSet::from([Effect::OutputMessage]),
        "call" => HashSet::from([Effect::Interaction]),
        "mint" => HashSet::from([Effect::MintAsset]),
        "burn" => HashSet::from([Effect::BurnAsset]),
        // the rest of the assembly instructions are considered to not have effects
        _ => HashSet::new(),
    }
}

fn set_union<E>(set1: HashSet<E>, set2: HashSet<E>) -> HashSet<E>
where
    E: std::hash::Hash + Eq + Clone,
{
    set1.union(&set2).cloned().collect()
}

fn map_hashsets_union<T, F, E>(elems: &[T], to_set: F) -> HashSet<E>
where
    F: Fn(&T) -> HashSet<E>,
    E: std::hash::Hash + Eq + Clone,
{
    elems
        .iter()
        .fold(HashSet::new(), |set, e| set_union(to_set(e), set))
}

fn effects_of_codeblock(engines: &Engines, codeblock: &ty::TyCodeBlock) -> HashSet<Effect> {
    map_hashsets_union(&codeblock.contents, |entry| {
        effects_of_codeblock_entry(engines, entry)
    })
}

fn effects_of_expressions(engines: &Engines, exprs: &[ty::TyExpression]) -> HashSet<Effect> {
    map_hashsets_union(exprs, |e| effects_of_expression(engines, e))
}

fn effects_of_struct_expressions(
    engines: &Engines,
    struct_exprs: &[ty::TyStructExpressionField],
) -> HashSet<Effect> {
    map_hashsets_union(struct_exprs, |se| effects_of_expression(engines, &se.value))
}

fn effects_of_asm_ops(asm_ops: &[AsmOp]) -> HashSet<Effect> {
    map_hashsets_union(asm_ops, effects_of_asm_op)
}

fn effects_of_register_initializers(
    engines: &Engines,
    initializers: &[ty::TyAsmRegisterDeclaration],
) -> HashSet<Effect> {
    map_hashsets_union(initializers, |asm_reg_decl| {
        asm_reg_decl
            .initializer
            .as_ref()
            .map_or(HashSet::new(), |e| effects_of_expression(engines, e))
    })
}
