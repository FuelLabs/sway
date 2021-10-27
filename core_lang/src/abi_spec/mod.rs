mod ast_node;

use crate::{
    error::{err, CompileResult},
    ParseTree,
};

use serde_json::{json, Value};

pub fn generate_abi_spec<'sc>(parse_tree: ParseTree<'sc>) -> CompileResult<'sc, Value> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let mut json_nodes = vec![];
    for node in parse_tree.root_nodes {
        json_nodes.push(check!(
            ast_node::generate_abi_spec(node),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }
    let json_tree = json!(json_nodes);

    unimplemented!()
}
