library constants;

const ETH_ID = 0x0000000000000000000000000000000000000000000000000000000000000000;
const ZERO = 0x0000000000000000000000000000000000000000000000000000000000000000;
const MAX_U64 = 0x000000000000000000000000000000000000000000000000FFFFFFFFFFFFFFFF;
const MAX_B256 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
// @todo tx format may change, in which case the magic number "384" must be changed.
// TransactionScript outputsCount has a 6 word/384-bit offset
const OUTPUT_LENGTH_LOCATION = 384;
// output types: https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/compressed_tx_format.md#output
const OUTPUT_VARIABLE_TYPE = 4;