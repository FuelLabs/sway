use crate::{error::CompileResult, AstNode};

use serde_json::Value;

pub fn generate_abi_spec<'sc>(node: AstNode<'sc>) -> CompileResult<'sc, Value> {
    unimplemented!()
}
