use crate::language::ty;

#[derive(Clone, Debug)]
pub struct ContractCallParams {
    pub(crate) func_selector: [u8; 4],
    pub(crate) contract_address: Box<ty::TyExpression>,
}
