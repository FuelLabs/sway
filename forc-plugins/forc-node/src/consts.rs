/// Minimum fuel-core version supported.
pub const MIN_FUEL_CORE_VERSION: &str = "0.43.0";

pub const MINIMUM_OPEN_FILE_DESCRIPTOR_LIMIT: u64 = 51200;

pub const TESTNET_SERVICE_NAME: &str = "fuel-sepolia-testnet-node";
pub const TESTNET_SYNC_HEADER_BATCH_SIZE: u32 = 100;
pub const TESTNET_RELAYER_LISTENING_CONTRACT: &str = "0x01855B78C1f8868DE70e84507ec735983bf262dA";
pub const TESTNET_RELAYER_DA_DEPLOY_HEIGHT: u32 = 5827607;
pub const TESTNET_RELAYER_LOG_PAGE_SIZE: u32 = 500;
pub const TESTNET_SYNC_BLOCK_STREAM_BUFFER_SIZE: u32 = 30;
pub const TESTNET_BOOTSTRAP_NODE: &str = "/dnsaddr/testnet.fuel.network.";

pub const MAINNET_BOOTSTRAP_NODE: &str = "/dnsaddr/mainnet.fuel.network.";
pub const MAINNET_SERVICE_NAME: &str = "fuel-mainnet-node";
pub const MAINNET_SYNC_HEADER_BATCH_SIZE: u32 = 30;
pub const MAINNET_RELAYER_LISTENING_CONTRACT: &str = "0xAEB0c00D0125A8a788956ade4f4F12Ead9f65DDf";
pub const MAINNET_RELAYER_DA_DEPLOY_HEIGHT: u32 = 20620434;
pub const MAINNET_RELAYER_LOG_PAGE_SIZE: u32 = 100;

/// Name of the folder for testnet at the configuration repo:
/// https://github.com/fuelLabs/chain-configuration/
/// And name of the db path if persistent db is used.
pub const TESTNET_CONFIG_FOLDER_NAME: &str = "ignition-test";
/// Name of the folder for ignition mainnet at the configuration repo:
/// https://github.com/fuelLabs/chain-configuration/
/// And name of the db path if persistent db is used.
pub const IGNITION_CONFIG_FOLDER_NAME: &str = "ignition";
/// Name of the folder for local configuration repo:
/// And name of the db path if persistent db is used.
pub const LOCAL_CONFIG_FOLDER_NAME: &str = "local";
/// Name of the github repository that hosts chain-configurations.
pub const CHAIN_CONFIG_REPO_NAME: &str = "chain-configuration";

pub const DEFAULT_PORT: u16 = 4000;
pub const DEFAULT_PEERING_PORT: u16 = 30333;

pub const CONFIG_FOLDER: &str = "chainspecs";
pub const DB_FOLDER: &str = "db";
