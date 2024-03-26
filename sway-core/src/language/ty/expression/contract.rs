use crate::language::ty::*;

#[derive(Clone, Debug, deepsize::DeepSizeOf)]
pub struct ContractCallParams {
    pub(crate) func_selector: [u8; 4],
    pub(crate) contract_address: Box<TyExpression>,
    pub(crate) contract_caller: Box<TyExpression>,
}
