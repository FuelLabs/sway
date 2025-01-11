use crate::language::ty::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractCallParams {
    // This is none in encoding V1
    pub(crate) func_selector: Option<[u8; 4]>,
    pub(crate) contract_address: Box<TyExpression>,
    pub(crate) contract_caller: Box<TyExpression>,
}
