library constants;
use ::contract_id::ContractId;

// TODO: use ZERO_B256 const in BASE_ASSET_ID initialization when https://github.com/FuelLabs/sway/issues/2151 will be resolved
const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const BASE_ASSET_ID = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
