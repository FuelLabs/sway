library evm_test_abi;

use std::vm::evm::evm_address::EvmAddress;

abi EvmTest {
    fn evm_address_from_literal() -> EvmAddress;
    fn evm_address_from_argument(raw_address: b256) -> EvmAddress;
}
