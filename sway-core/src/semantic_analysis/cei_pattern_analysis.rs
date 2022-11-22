// CEI stands for "Checks, Effects, Interactions".  We check that no storage writes
// (effects) occur after calling external contracts (interaction) and issue warnings
// if it's the case.
// See this [blog post](https://fravoll.github.io/solidity-patterns/checks_effects_interactions.html)
// for more detail on vulnerabilities in case of storage modification after interaction
// and this [blog post](https://chainsecurity.com/curve-lp-oracle-manipulation-post-mortem)
// for more information on storage reads after interaction.

use crate::{
    declaration_engine::DeclarationId,
    language::{
        ty::{self, TyFunctionDeclaration},
        AsmOp,
    },
    look_up_type_id,
};
use std::collections::HashSet;
use sway_error::warning::{CompileWarning, Warning};
use sway_types::{Ident, Span, Spanned};

#[derive(PartialEq, Eq, Hash, Clone)]
enum Effect {
    Interaction,  // interaction with external contracts
    StorageWrite, // storage modification
    StorageRead,  // storage read
}

// The algorithm that searches for storage operations after interaction
// is organized as an automaton.
// After an interaction is found in a code block, we keep looking either for
// storage reads or writes.
enum CEIAnalysisState {
    LookingForInteraction, // initial state of the automaton
    LookingForStorageReadOrWrite,
}

pub(crate) fn analyze_program(prog: &ty::TyProgram) -> Vec<CompileWarning> {
    match &prog.kind {
        // Libraries, scripts, or predicates can't access storage
        // so we don't analyze these
        ty::TyProgramKind::Library { .. }
        | ty::TyProgramKind::Script { .. }
        | ty::TyProgramKind::Predicate { .. } => vec![],
        ty::TyProgramKind::Contract { .. } => analyze_contract(&prog.root.all_nodes),
    }
}

fn analyze_contract(ast_nodes: &[ty::TyAstNode]) -> Vec<CompileWarning> {
    let mut warnings: Vec<CompileWarning> = vec![];
    for fn_decl in contract_entry_points(ast_nodes) {
        analyze_code_block(&fn_decl.body, &fn_decl.name, &mut warnings);
    }
    warnings
}

// standalone functions and methods
fn contract_entry_points(ast_nodes: &[ty::TyAstNode]) -> Vec<ty::TyFunctionDeclaration> {
    use crate::ty::TyAstNodeContent::Declaration;
    ast_nodes
        .iter()
        .flat_map(|ast_node| match &ast_node.content {
            Declaration(ty::TyDeclaration::FunctionDeclaration(decl_id)) => {
                decl_id_to_fn_decls(decl_id, &ast_node.span)
            }
            Declaration(ty::TyDeclaration::ImplTrait(decl_id)) => {
                impl_trait_methods(decl_id, &ast_node.span)
            }
            _ => vec![],
        })
        .collect()
}

fn decl_id_to_fn_decls(decl_id: &DeclarationId, span: &Span) -> Vec<TyFunctionDeclaration> {
    use crate::declaration_engine::de_get_function;
    de_get_function(decl_id.clone(), span).map_or(vec![], |fn_decl| vec![fn_decl])
}

fn impl_trait_methods<'a>(
    impl_trait_decl_id: &'a DeclarationId,
    span: &'a Span,
) -> Vec<ty::TyFunctionDeclaration> {
    use crate::declaration_engine::de_get_impl_trait;
    match de_get_impl_trait(impl_trait_decl_id.clone(), span) {
        Ok(impl_trait) => impl_trait
            .methods
            .iter()
            .flat_map(|fn_decl| decl_id_to_fn_decls(fn_decl, span))
            .collect(),
        Err(_) => vec![],
    }
}

// This is the main part of the analysis algorithm:
// we are looking for state effects after contract interaction
fn analyze_code_block(
    codeblock: &ty::TyCodeBlock,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    let mut interaction_span: Span = Span::dummy();
    let mut codeblock_effects = HashSet::new();
    let mut analysis_state: CEIAnalysisState = CEIAnalysisState::LookingForInteraction;

    for ast_node in &codeblock.contents {
        let codeblock_entry_effects = analyze_code_block_entry(ast_node, block_name, warnings);
        match analysis_state {
            CEIAnalysisState::LookingForInteraction => {
                if codeblock_entry_effects.contains(&Effect::Interaction) {
                    analysis_state = CEIAnalysisState::LookingForStorageReadOrWrite;
                    interaction_span = ast_node.span.clone();
                }
            }
            CEIAnalysisState::LookingForStorageReadOrWrite => warn_after_interaction(
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
    entry: &ty::TyAstNode,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    match &entry.content {
        ty::TyAstNodeContent::Declaration(decl) => {
            analyze_codeblock_decl(decl, block_name, warnings)
        }
        ty::TyAstNodeContent::Expression(expr)
        | ty::TyAstNodeContent::ImplicitReturnExpression(expr) => {
            analyze_expression(expr, block_name, warnings)
        }
        ty::TyAstNodeContent::SideEffect => HashSet::new(),
    }
}

fn analyze_codeblock_decl(
    decl: &ty::TyDeclaration,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    // Declarations (except variable declarations) are not allowed in a codeblock
    use crate::ty::TyDeclaration::*;
    match decl {
        VariableDeclaration(var_decl) => analyze_expression(&var_decl.body, block_name, warnings),
        _ => HashSet::new(),
    }
}

fn analyze_expression(
    expr: &ty::TyExpression,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    use crate::ty::TyExpressionVariant::*;
    match &expr.expression {
        // base cases: no warnings can be emitted
        Literal(_)
        | VariableExpression { .. }
        | FunctionParameter
        | StorageAccess(_)
        | Break
        | Continue
        | AbiName(_) => effects_of_expression(expr),
        Reassignment(reassgn) => analyze_expression(&reassgn.rhs, block_name, warnings),
        StorageReassignment(reassgn) => {
            let storage_effs = HashSet::from([Effect::StorageWrite]);
            let rhs_effs = analyze_expression(&reassgn.rhs, block_name, warnings);
            if rhs_effs.contains(&Effect::Interaction) {
                warn_after_interaction(
                    &storage_effs,
                    &reassgn.rhs.span,
                    &expr.span,
                    block_name,
                    warnings,
                )
            };
            set_union(storage_effs, rhs_effs)
        }
        CodeBlock(codeblock) => analyze_code_block(codeblock, block_name, warnings),
        LazyOperator {
            lhs: left,
            rhs: right,
            ..
        }
        | ArrayIndex {
            prefix: left,
            index: right,
        } => analyze_two_expressions(left, right, block_name, warnings),
        FunctionApplication {
            arguments,
            function_decl_id,
            selector,
            call_path,
            ..
        } => {
            use crate::declaration_engine::de_get_function;
            let func = de_get_function(function_decl_id.clone(), &expr.span).unwrap();
            // we don't need to run full analysis on the function body as it will be covered
            // as a separate step of the whole contract analysis
            // we just need function's effects at this point
            let fn_effs = effects_of_codeblock(&func.body);

            // assuming left-to-right arguments evaluation
            // we run CEI violation analysis as if the arguments form a code block
            let args_effs = analyze_expressions(
                arguments.iter().map(|(_, e)| e).collect(),
                block_name,
                warnings,
            );
            if args_effs.contains(&Effect::Interaction) {
                // TODO: interaction span has to be more precise and point to an argument which performs interaction
                let last_arg_span = &arguments.last().unwrap().1.span;
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
                analyze_expressions(intrinsic.arguments.iter().collect(), block_name, warnings);
            if args_effs.contains(&Effect::Interaction) {
                // TODO: interaction span has to be more precise and point to an argument which performs interaction
                warn_after_interaction(&intr_effs, &expr.span, &expr.span, block_name, warnings)
            }
            set_union(intr_effs, args_effs)
        }
        Tuple { fields: exprs } | Array { contents: exprs } => {
            // assuming left-to-right fields/elements evaluation
            analyze_expressions(exprs.iter().collect(), block_name, warnings)
        }
        StructExpression { fields, .. } => {
            // assuming left-to-right fields evaluation
            analyze_expressions(
                fields.iter().map(|e| &e.value).collect(),
                block_name,
                warnings,
            )
        }
        StructFieldAccess { prefix: expr, .. }
        | TupleElemAccess { prefix: expr, .. }
        | Return(expr)
        | EnumTag { exp: expr }
        | UnsafeDowncast { exp: expr, .. }
        | AbiCast { address: expr, .. } => analyze_expression(expr, block_name, warnings),
        EnumInstantiation { contents, .. } => match contents {
            Some(expr) => analyze_expression(expr, block_name, warnings),
            None => HashSet::new(),
        },
        IfExp {
            condition,
            then,
            r#else,
        } => {
            let cond_then_effs = analyze_two_expressions(condition, then, block_name, warnings);
            let cond_else_effs = match r#else {
                Some(else_exp) => {
                    analyze_two_expressions(condition, else_exp, block_name, warnings)
                }
                None => HashSet::new(),
            };
            set_union(cond_then_effs, cond_else_effs)
        }
        WhileLoop { condition, body } => {
            // if the loop (condition + body) contains both interaction and storage operations
            // in _any_ order, we report CEI pattern violation
            let cond_effs = analyze_expression(condition, block_name, warnings);
            let body_effs = analyze_code_block(body, block_name, warnings);
            let res_effs = set_union(cond_effs, body_effs);
            if res_effs.is_superset(&HashSet::from([Effect::Interaction, Effect::StorageRead])) {
                warnings.push(CompileWarning {
                    span: expr.span.clone(),
                    warning_content: Warning::StorageReadAfterInteraction {
                        block_name: block_name.clone(),
                    },
                });
            };
            if res_effs.is_superset(&HashSet::from([Effect::Interaction, Effect::StorageWrite])) {
                warnings.push(CompileWarning {
                    span: expr.span.clone(),
                    warning_content: Warning::StorageWriteAfterInteraction {
                        block_name: block_name.clone(),
                    },
                });
            };
            res_effs
        }
        AsmExpression {
            registers, body, ..
        } => {
            let init_exprs = registers
                .iter()
                .filter_map(|rdecl| rdecl.initializer.as_ref())
                .collect();
            let init_effs = analyze_expressions(init_exprs, block_name, warnings);
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
    first: &ty::TyExpression,
    second: &ty::TyExpression,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    let first_effs = analyze_expression(first, block_name, warnings);
    let second_effs = analyze_expression(second, block_name, warnings);
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
fn analyze_expressions(
    expressions: Vec<&ty::TyExpression>,
    block_name: &Ident,
    warnings: &mut Vec<CompileWarning>,
) -> HashSet<Effect> {
    let mut interaction_span: Span = Span::dummy();
    let mut accumulated_effects = HashSet::new();
    let mut analysis_state: CEIAnalysisState = CEIAnalysisState::LookingForInteraction;

    for expr in expressions {
        let expr_effs = analyze_expression(expr, block_name, warnings);
        match analysis_state {
            CEIAnalysisState::LookingForInteraction => {
                if expr_effs.contains(&Effect::Interaction) {
                    analysis_state = CEIAnalysisState::LookingForStorageReadOrWrite;
                    interaction_span = expr.span.clone();
                }
            }
            CEIAnalysisState::LookingForStorageReadOrWrite => warn_after_interaction(
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
                    analysis_state = CEIAnalysisState::LookingForStorageReadOrWrite;
                    interaction_span = asm_op.span.clone();
                }
            }
            CEIAnalysisState::LookingForStorageReadOrWrite => warn_after_interaction(
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
    if ast_node_effects.contains(&Effect::StorageRead) {
        warnings.push(CompileWarning {
            span: Span::join(interaction_span.clone(), effect_span.clone()),
            warning_content: Warning::StorageReadAfterInteraction {
                block_name: block_name.clone(),
            },
        });
    };
    if ast_node_effects.contains(&Effect::StorageWrite) {
        warnings.push(CompileWarning {
            span: Span::join(interaction_span.clone(), effect_span.clone()),
            warning_content: Warning::StorageWriteAfterInteraction {
                block_name: block_name.clone(),
            },
        });
    }
}

fn effects_of_codeblock_entry(ast_node: &ty::TyAstNode) -> HashSet<Effect> {
    match &ast_node.content {
        ty::TyAstNodeContent::Declaration(decl) => effects_of_codeblock_decl(decl),
        ty::TyAstNodeContent::Expression(expr)
        | ty::TyAstNodeContent::ImplicitReturnExpression(expr) => effects_of_expression(expr),
        ty::TyAstNodeContent::SideEffect => HashSet::new(),
    }
}

fn effects_of_codeblock_decl(decl: &ty::TyDeclaration) -> HashSet<Effect> {
    use crate::ty::TyDeclaration::*;
    match decl {
        VariableDeclaration(var_decl) => effects_of_expression(&var_decl.body),
        // Declarations (except variable declarations) are not allowed in the body of a function
        _ => HashSet::new(),
    }
}

fn effects_of_expression(expr: &ty::TyExpression) -> HashSet<Effect> {
    use crate::ty::TyExpressionVariant::*;
    match &expr.expression {
        Literal(_)
        | VariableExpression { .. }
        | FunctionParameter
        | Break
        | Continue
        | AbiName(_) => HashSet::new(),
        // this type of assignment only mutates local variables and not storage
        Reassignment(reassgn) => effects_of_expression(&reassgn.rhs),
        StorageAccess(_) => match look_up_type_id(expr.return_type) {
            // accessing a storage map's method (or a storage vector's method),
            // which is represented using a struct with empty fields
            // does not result in a storage read
            crate::TypeInfo::Struct { fields, .. } if fields.is_empty() => HashSet::new(),
            // if it's an empty enum then it cannot be constructed and hence cannot be read
            // adding this check here just to be on the safe side
            crate::TypeInfo::Enum { variant_types, .. } if variant_types.is_empty() => {
                HashSet::new()
            }
            _ => HashSet::from([Effect::StorageRead]),
        },
        StorageReassignment(storage_reassign) => {
            let mut effs = HashSet::from([Effect::StorageWrite]);
            effs.extend(effects_of_expression(&storage_reassign.rhs));
            effs
        }
        LazyOperator { lhs, rhs, .. }
        | ArrayIndex {
            prefix: lhs,
            index: rhs,
        } => {
            let mut effs = effects_of_expression(lhs);
            let rhs_effs = effects_of_expression(rhs);
            effs.extend(rhs_effs);
            effs
        }
        Tuple { fields: exprs } | Array { contents: exprs } => effects_of_expressions(exprs),
        StructExpression { fields, .. } => effects_of_struct_expressions(fields),
        CodeBlock(codeblock) => effects_of_codeblock(codeblock),
        IfExp {
            condition,
            then,
            r#else,
        } => {
            let mut effs = effects_of_expression(condition);
            effs.extend(effects_of_expression(then));
            let else_effs = match r#else {
                Some(expr) => effects_of_expression(expr),
                None => HashSet::new(),
            };
            effs.extend(else_effs);
            effs
        }
        StructFieldAccess { prefix: expr, .. }
        | TupleElemAccess { prefix: expr, .. }
        | EnumTag { exp: expr }
        | UnsafeDowncast { exp: expr, .. }
        | Return(expr) => effects_of_expression(expr),
        EnumInstantiation { contents, .. } => match contents {
            Some(expr) => effects_of_expression(expr),
            None => HashSet::new(),
        },
        AbiCast { address, .. } => effects_of_expression(address),
        IntrinsicFunction(intr_fn) => effects_of_expressions(&intr_fn.arguments)
            .union(&effects_of_intrinsic(&intr_fn.kind))
            .cloned()
            .collect(),
        WhileLoop { condition, body } => effects_of_expression(condition)
            .union(&effects_of_codeblock(body))
            .cloned()
            .collect(),
        FunctionApplication {
            function_decl_id,
            arguments,
            selector,
            ..
        } => {
            use crate::declaration_engine::de_get_function;
            let fn_body = de_get_function(function_decl_id.clone(), &expr.span)
                .unwrap()
                .body;
            let mut effs = effects_of_codeblock(&fn_body);
            let args_effs = map_hashsets_union(arguments, |e| effects_of_expression(&e.1));
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
        } => effects_of_register_initializers(registers)
            .union(&effects_of_asm_ops(body))
            .cloned()
            .collect(),
    }
}

fn effects_of_intrinsic(intr: &sway_ast::Intrinsic) -> HashSet<Effect> {
    use sway_ast::Intrinsic::*;
    match intr {
        StateStoreWord | StateStoreQuad => HashSet::from([Effect::StorageWrite]),
        StateLoadWord | StateLoadQuad => HashSet::from([Effect::StorageRead]),
        Revert | IsReferenceType | SizeOfType | SizeOfVal | Eq | Gtf | AddrOf | Log | Add | Sub
        | Mul | Div | PtrAdd | PtrSub | GetStorageKey => HashSet::new(),
    }
}

fn effects_of_asm_op(op: &AsmOp) -> HashSet<Effect> {
    match op.op_name.as_str().to_lowercase().as_str() {
        "sww" | "swwq" => HashSet::from([Effect::StorageWrite]),
        "srw" | "srwq" | "bal" => HashSet::from([Effect::StorageRead]),
        "call" => HashSet::from([Effect::Interaction]),
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

fn effects_of_codeblock(codeblock: &ty::TyCodeBlock) -> HashSet<Effect> {
    map_hashsets_union(&codeblock.contents, effects_of_codeblock_entry)
}

fn effects_of_expressions(exprs: &[ty::TyExpression]) -> HashSet<Effect> {
    map_hashsets_union(exprs, effects_of_expression)
}

fn effects_of_struct_expressions(struct_exprs: &[ty::TyStructExpressionField]) -> HashSet<Effect> {
    map_hashsets_union(struct_exprs, |se| effects_of_expression(&se.value))
}

fn effects_of_asm_ops(asm_ops: &[AsmOp]) -> HashSet<Effect> {
    map_hashsets_union(asm_ops, effects_of_asm_op)
}

fn effects_of_register_initializers(
    initializers: &[ty::TyAsmRegisterDeclaration],
) -> HashSet<Effect> {
    map_hashsets_union(initializers, |asm_reg_decl| {
        asm_reg_decl
            .initializer
            .as_ref()
            .map_or(HashSet::new(), effects_of_expression)
    })
}
