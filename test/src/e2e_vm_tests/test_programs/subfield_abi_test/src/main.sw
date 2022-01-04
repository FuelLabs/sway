script;

const ETH_ID = 0x0000000000000000000000000000000000000000000000000000000000000000;

/// Parameters for `transfer_to_output` function.
pub struct ParamsTransferToOutput {
    coins: u64,
}

/// Parameters for `force_transfer` function.
pub struct ParamsForceTransfer {
    coins: u64,
}

abi TestFuelCoin {
    fn mint(gas: u64, coins: u64, token_id: b256, mint_amount: u64);
}

struct Opts {
    id: ContractId,
    gas: u64,
    coins: u64,
}
/// The ContractId type, a struct wrappper around the inner `value`.
pub struct ContractId {
    value: b256,
}

// @todo make this generic when possible
pub trait From {
    fn from(b: b256) -> Self;
} {
    fn into(addr: ContractId) -> b256 {
        addr.value
    }
}

/// Functions for casting between the b256 and ContractId types.
impl From for ContractId {
    fn from(bits: b256) -> ContractId {
        ContractId {
            value: bits,
        }
    }
}

fn main() -> bool {
    let default = Opts {
        gas: 1000,
        coins: 0,
        id: ~ContractId::from(ETH_ID),
    };

    // the already deployed balance_test contract

    // the deployed fuel_coin contract
    let fuelcoin_id = ~ContractId::from(0x9c8a446c98b85592823934520a4865a5a93b8dbb0e825e98ef26a08a6e88a17b);
    // @todo use correct type ContractId
    let fuel_coin = abi(TestFuelCoin, fuelcoin_id.value);

    fuel_coin.mint(default.gas, default.coins, default.id.value, 11);
    true
}
