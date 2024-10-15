/// Default to localhost to favour the common case of testing.
pub const NODE_URL: &str = sway_utils::constants::DEFAULT_NODE_URL;
pub const TESTNET_ENDPOINT_URL: &str = "https://testnet.fuel.network";
pub const MAINNET_ENDPOINT_URL: &str = "https://mainnet.fuel.network";

pub const TESTNET_FAUCET_URL: &str = "https://faucet-testnet.fuel.network";

pub const TESTNET_EXPLORER_URL: &str = "https://app-testnet.fuel.network";
pub const MAINNET_EXPLORER_URL: &str = "https://app.fuel.network";

/// Default PrivateKey to sign transactions submitted to local node.
pub const DEFAULT_PRIVATE_KEY: &str =
    "0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c";
/// The maximum time to wait for a transaction to be included in a block by the node
pub const TX_SUBMIT_TIMEOUT_MS: u64 = 30_000u64;
