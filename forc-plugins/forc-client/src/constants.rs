/// Default to localhost to favour the common case of testing.
pub const NODE_URL: &str = sway_utils::constants::DEFAULT_NODE_URL;
pub const BETA_2_ENDPOINT_URL: &str = "https://node-beta-2.fuel.network";
pub const BETA_3_ENDPOINT_URL: &str = "https://beta-3.fuel.network";
pub const BETA_4_ENDPOINT_URL: &str = "https://beta-4.fuel.network";
pub const BETA_5_ENDPOINT_URL: &str = "https://beta-5.fuel.network";
pub const DEVNET_ENDPOINT_URL: &str = "https://devnet.fuel.network";
pub const TESTNET_ENDPOINT_URL: &str = "https://testnet.fuel.network";

pub const BETA_2_FAUCET_URL: &str = "https://faucet-beta-2.fuel.network";
pub const BETA_3_FAUCET_URL: &str = "https://faucet-beta-3.fuel.network";
pub const BETA_4_FAUCET_URL: &str = "https://faucet-beta-4.fuel.network";
pub const BETA_5_FAUCET_URL: &str = "https://faucet-beta-5.fuel.network";
pub const DEVNET_FAUCET_URL: &str = "https://faucet-devnet.fuel.network";
pub const TESTNET_FAUCET_URL: &str = "https://faucet-testnet.fuel.network";

pub const TESTNET_EXPLORER_URL: &str = "https://app.fuel.network";

/// Default PrivateKey to sign transactions submitted to local node.
pub const DEFAULT_PRIVATE_KEY: &str =
    "0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c";
/// The maximum time to wait for a transaction to be included in a block by the node
pub const TX_SUBMIT_TIMEOUT_MS: u64 = 30_000u64;
