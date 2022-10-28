// CEI stands for "Checks, Effects, Interactions".  We check that no storage writes
// (effects) occur after calling external contracts (interaction) and issue warnings
// if it's the case.
// See this [blog post](https://fravoll.github.io/solidity-patterns/checks_effects_interactions.html)
// for more detail.

use crate::{
    declaration_engine::DeclarationId,
    language::ty::{self, TyFunctionDeclaration},
};
use std::collections::HashSet;
use sway_error::warning::{CompileWarning, Warning};
use sway_types::Span;

#[derive(PartialEq, Eq, Hash, Clone)]
enum Effect {
    Interaction,  // interaction with external contracts
    StorageWrite, // storage modification
}

enum CEIAnalysisState {
    LookingForInteraction,
    LookingForStorageWrite,
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
    contract_entry_points(ast_nodes)
        .iter()
        .flat_map(|fn_decl| analyze_code_block(&fn_decl.body))
        .collect()
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
fn analyze_code_block(code_block: &ty::TyCodeBlock) -> Vec<CompileWarning> {
    let mut warnings: Vec<CompileWarning> = vec![];
    let mut interaction_span: Span = Span::dummy();
    let mut analysis_state: CEIAnalysisState = CEIAnalysisState::LookingForInteraction;

    for ast_node in &code_block.contents {
        match analysis_state {
            CEIAnalysisState::LookingForInteraction => {
                if effects_of_codeblock_entry(ast_node).contains(&Effect::Interaction) {
                    analysis_state = CEIAnalysisState::LookingForStorageWrite;
                    interaction_span = ast_node.span.clone();
                }
            }
            CEIAnalysisState::LookingForStorageWrite => {
                if effects_of_codeblock_entry(ast_node).contains(&Effect::StorageWrite) {
                    warnings.push(CompileWarning {
                        span: Span::join(interaction_span, ast_node.span.clone()),
                        warning_content: Warning::StorageWriteAfterInteraction,
                    });
                    return warnings;
                }
            }
        }
    }
    warnings
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
        VariableDeclaration(var_decl) => effects_of_expression(&(*var_decl).body),
        // Declarations (except variable declarations) are not allowed in the body of a function
        _ => HashSet::new(),
    }
}

fn effects_of_expression(expr: &ty::TyExpression) -> HashSet<Effect> {
    use crate::ty::TyExpressionVariant::*;
    match &expr.expression {
        Literal(_)
        | VariableExpression{..}
        | FunctionParameter
        | Break
        | Continue
        // this type of assignment only mutates local variables and not storage
        | Reassignment(_)
        | AbiName(_)
        | StorageAccess(_) => HashSet::new(),
        StorageReassignment(storage_reassign) => {
            let mut effs = HashSet::from([Effect::StorageWrite]);
            effs.extend(effects_of_expression(&storage_reassign.rhs));
            effs
        },
        LazyOperator {lhs, rhs, .. }
        | ArrayIndex { prefix: lhs, index: rhs } => {
            let mut effs = effects_of_expression(lhs);
            let rhs_effs = effects_of_expression(rhs);
            effs.extend(rhs_effs);
            effs
        },
        Tuple { fields: exprs }
        | Array { contents: exprs } => {
            effects_of_expressions(exprs)
        },
        StructExpression {fields, ..} => {
            effects_of_struct_expressions(fields)
        },
        CodeBlock(codeblock) => {
            effects_of_codeblock(codeblock)
        },
        IfExp { condition, then, r#else } => {
            let mut effs = effects_of_expression(condition);
            effs.extend(effects_of_expression(then));
            let else_effs =
                match r#else {
                    Some(expr) => effects_of_expression(expr),
                    None => HashSet::new(),
                };
            effs.extend(else_effs);
            effs
        },
        StructFieldAccess { prefix: expr, ..}
        | TupleElemAccess { prefix: expr, ..}
        | EnumTag { exp: expr }
        | UnsafeDowncast {exp: expr, ..}
        | Return(expr) => {
            effects_of_expression(expr)
        },
        EnumInstantiation {contents, ..} => {
            match contents {
                Some(expr) => effects_of_expression(expr),
                None => HashSet::new(),
            }
        },
        AbiCast {address, ..} => {
            effects_of_expression(address)
        },
        IntrinsicFunction(intr_fn) => {
            effects_of_expressions(&intr_fn.arguments)
            .union(&effects_of_intrinsic(&intr_fn.kind))
            .cloned()
            .collect()
        },
        WhileLoop { condition, body } => {
            effects_of_expression(condition)
            .union(&effects_of_codeblock(body))
            .cloned()
            .collect()
        },
        FunctionApplication {function_decl_id,
                             arguments,
                             selector, ..} => {

            use crate::declaration_engine::de_get_function;
            let fn_body = de_get_function(function_decl_id.clone(), &expr.span).unwrap().body;
            let mut effs = effects_of_codeblock(&fn_body);
            let args_effs = map_hashsets_union(arguments, |e| effects_of_expression(&e.1));
            effs.extend(args_effs);
            if selector.is_some() {
                // external contract call (a.k.a. interaction)
                effs.extend(HashSet::from([Effect::Interaction]))
            };
            effs
        },
        // TODO: process assembly blocks
        AsmExpression {
            registers: _,
            body: _,
            whole_block_span: _,
            ..
        } => {
            // temporary solution, will remove after this is finished
            HashSet::new()
        },
    }
}

fn effects_of_intrinsic(intr: &sway_ast::Intrinsic) -> HashSet<Effect> {
    use sway_ast::Intrinsic::*;
    match intr {
        StateStoreWord | StateStoreQuad => HashSet::from([Effect::StorageWrite]),
        // TODO: figure out the effect of __revert
        Revert | GetStorageKey | IsReferenceType | SizeOfType | SizeOfVal | Eq | Gtf | AddrOf
        | StateLoadWord | StateLoadQuad | Log | Add | Sub | Mul | Div => HashSet::new(),
    }
}

fn map_hashsets_union<T, F, E>(elems: &[T], to_set: F) -> HashSet<E>
where
    F: Fn(&T) -> HashSet<E>,
    E: std::hash::Hash + Eq + Clone,
{
    elems.iter().fold(HashSet::new(), |set, e| {
        to_set(e).union(&set).cloned().collect()
    })
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
