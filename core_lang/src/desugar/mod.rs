mod ast_node;
mod code_block;
mod declaration;
mod expression;
mod matcher;

use crate::error::{err, ok, CompileResult};
use crate::semantic_analysis::{TreeType, TypedParseTree};

/*
pub fn desugar<'sc>(tree: HllParseTree<'sc>) -> CompileResult<'sc, HllParseTree<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut desugar = |ast: Option<_>| {
        ast.map(|tree| desugar_parse_tree(tree).ok(&mut warnings, &mut errors))
            .flatten()
    };
    let tree = HllParseTree {
        script_ast: desugar(tree.script_ast),
        predicate_ast: desugar(tree.predicate_ast),
        contract_ast: desugar(tree.contract_ast),
        library_exports: tree.library_exports,
    };
    ok(tree, warnings, errors)
}

fn desugar_parse_tree<'sc>(parse_tree: ParseTree<'sc>) -> CompileResult<'sc, ParseTree<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut new_root_nodes = vec![];
    for root_node in parse_tree.root_nodes.into_iter() {
        new_root_nodes.push(check!(
            desugar_ast_node(root_node),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }
    let parse_tree = ParseTree {
        root_nodes: new_root_nodes,
        span: parse_tree.span,
    };
    ok(parse_tree, warnings, errors)
}
*/
