use crate::error::{err, ok, CompileResult};
use crate::CodeBlock;

use super::ast_node::desugar_ast_node;

pub fn desugar_code_block<'sc>(block: CodeBlock<'sc>) -> CompileResult<'sc, CodeBlock<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut desugared_contents = vec![];
    for node in block.contents.into_iter() {
        desugared_contents.push(check!(
            desugar_ast_node(node),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }
    let block = CodeBlock {
        whole_block_span: block.whole_block_span,
        contents: desugared_contents,
    };
    ok(block, warnings, errors)
}
