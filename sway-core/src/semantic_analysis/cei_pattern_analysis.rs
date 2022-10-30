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
};
use std::collections::HashSet;
use sway_error::warning::{CompileWarning, Warning};
use sway_types::{Ident, Span};

#[derive(PartialEq, Eq, Hash, Clone)]
enum Effect {
    Interaction,  // interaction with external contracts
    StorageWrite, // storage modification
    StorageRead,  // storage read
}

// The algorithm that searches for storage operations after interaction
// is organized as an automaton.
// After an interaction is found in a code block, we keep looking either for
// storage reads or writes. After either one is found, we look for the opposite
// storage operation: e.g. if a read-after-interaction is found we search for a
// write-after-interaction, but we don't report mutliple violations of the same
// type per code block.
// After both kinds of the CEI pattern violation are found, we stop immediately.
enum CEIAnalysisState {
    LookingForInteraction, // initial state of the automaton
    LookingForStorageReadOrWrite,
    FoundWriteLookingForStorageRead, // a storage write is already found
    FoundReadLookingForStorageWrite, // a storage read is already found
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
        .flat_map(|fn_decl| analyze_code_block(&fn_decl.body, &fn_decl.name))
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
fn analyze_code_block(code_block: &ty::TyCodeBlock, block_name: &Ident) -> Vec<CompileWarning> {
    let mut warnings: Vec<CompileWarning> = vec![];
    let mut interaction_span: Span = Span::dummy();
    let mut analysis_state: CEIAnalysisState = CEIAnalysisState::LookingForInteraction;

    for ast_node in &code_block.contents {
        match analysis_state {
            CEIAnalysisState::LookingForInteraction => {
                if effects_of_codeblock_entry(ast_node).contains(&Effect::Interaction) {
                    analysis_state = CEIAnalysisState::LookingForStorageReadOrWrite;
                    interaction_span = ast_node.span.clone();
                }
            }
            CEIAnalysisState::LookingForStorageReadOrWrite => {
                let does_storage_write =
                    effects_of_codeblock_entry(ast_node).contains(&Effect::StorageWrite);
                let does_storage_read =
                    effects_of_codeblock_entry(ast_node).contains(&Effect::StorageRead);
                if does_storage_read {
                    warn_on_storage_read(&mut warnings, ast_node, &interaction_span, block_name)
                };
                if does_storage_write {
                    warn_on_storage_write(&mut warnings, ast_node, &interaction_span, block_name)
                }
                // compute the next automaton state
                if does_storage_read && does_storage_write {
                    // we are done: this statement does both: read and write to storage
                    return warnings;
                } else if does_storage_write {
                    analysis_state = CEIAnalysisState::FoundWriteLookingForStorageRead;
                } else if does_storage_read {
                    analysis_state = CEIAnalysisState::FoundReadLookingForStorageWrite;
                }
            }
            CEIAnalysisState::FoundReadLookingForStorageWrite => {
                // a storage read is already found at this point
                if effects_of_codeblock_entry(ast_node).contains(&Effect::StorageWrite) {
                    warn_on_storage_write(&mut warnings, ast_node, &interaction_span, block_name);
                    return warnings;
                }
            }
            CEIAnalysisState::FoundWriteLookingForStorageRead => {
                // a storage write is already found at this point
                if effects_of_codeblock_entry(ast_node).contains(&Effect::StorageRead) {
                    warn_on_storage_read(&mut warnings, ast_node, &interaction_span, block_name);
                    return warnings;
                }
            }
        }
    }
    warnings
}

fn warn_on_storage_read(
    warnings: &mut Vec<CompileWarning>,
    ast_node: &ty::TyAstNode,
    interaction_span: &Span,
    block_name: &Ident,
) {
    if effects_of_codeblock_entry(ast_node).contains(&Effect::StorageRead) {
        warnings.push(CompileWarning {
            span: Span::join(interaction_span.clone(), ast_node.span.clone()),
            warning_content: Warning::StorageReadAfterInteraction {
                block_name: block_name.clone(),
            },
        });
    }
}

fn warn_on_storage_write(
    warnings: &mut Vec<CompileWarning>,
    ast_node: &ty::TyAstNode,
    interaction_span: &Span,
    block_name: &Ident,
) {
    if effects_of_codeblock_entry(ast_node).contains(&Effect::StorageWrite) {
        warnings.push(CompileWarning {
            span: Span::join(interaction_span.clone(), ast_node.span.clone()),
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
        | AbiName(_) => HashSet::new(),
        StorageAccess(_) => HashSet::from([Effect::StorageRead]),
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
        AsmExpression {
            registers,
            body,
            whole_block_span: _,
            ..
        } => {
            effects_of_register_initializers(registers)
            .union(&effects_of_asm_ops(body))
            .cloned()
            .collect()
        },
    }
}

fn effects_of_intrinsic(intr: &sway_ast::Intrinsic) -> HashSet<Effect> {
    use sway_ast::Intrinsic::*;
    match intr {
        StateStoreWord | StateStoreQuad => HashSet::from([Effect::StorageWrite]),
        StateLoadWord | StateLoadQuad | GetStorageKey => HashSet::from([Effect::StorageRead]),
        Revert | IsReferenceType | SizeOfType | SizeOfVal | Eq | Gtf | AddrOf | Log | Add | Sub
        | Mul | Div => HashSet::new(),
    }
}

fn effects_of_asm_op(op: &AsmOp) -> HashSet<Effect> {
    match op.op_name.as_str().to_lowercase().as_str() {
        "sww" | "swwq" => HashSet::from([Effect::StorageWrite]),
        "srw" | "srwq" | "bal" => HashSet::from([Effect::StorageRead]),
        // the rest of the assembly instructions are considered to not have effects
        _ => HashSet::new(),
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
