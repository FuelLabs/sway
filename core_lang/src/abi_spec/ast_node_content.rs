use crate::{
    AstNodeContent,
    error::CompileResult
};

use serde_json::Value;

pub fn generate_abi_spec<'sc>(node: AstNodeContent<'sc>) -> CompileResult<'sc, Value> {
    unimplemented!()
}