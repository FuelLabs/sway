library abi;

use ::contract_id::ContractId;

pub fn contract_at(contract_abi : ContractCaller<_>, id: ContractId) -> ContractCaller<_> {
  let caller = abi(contract_abi, id.value);
  caller
}
