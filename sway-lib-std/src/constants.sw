library constants;

use ::{
  address::Address,
  contract_id::ContractId,
  identity::Identity,
};

pub const BASE_ASSET_ID = ~ContractId::from(ZERO_B256);
pub const ZERO_ADDRESS = ~Address::from(ZERO_B256);
pub const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
pub const ZERO_IDENTITY = ~Identity::Address(ZERO_ADDRESS);
