library constants;
use ::contract_id::ContractId;

// TODO: use ZERO_B256 const in BASE_ASSET_ID initialization when https://github.com/FuelLabs/sway/issues/2151 will be resolved
const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const BASE_ASSET_ID = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);

// Input types
const INPUT_COIN = 0u8;
const INPUT_CONTRACT = 1u8;
const INPUT_MESSAGE = 2u8;

// Output types
const OUTPUT_CONTRACT = 1u8;
const OUTPUT_MESSAGE = 2u8;
const OUTPUT_CHANGE = 3u8;
const OUTPUT_VARIABLE = 4u8;
const OUTPUT_CONTRACT_CREATED = 5u8;
