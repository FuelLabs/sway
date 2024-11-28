/// Minimum fuel-core version supported.
pub const MIN_FUEL_CORE_VERSION: &str = "0.40.0";

pub const TESTNET_RESERVED_NODE: &str = "/dns4/p2p-testnet.fuel.network/tcp/30333/p2p/16Uiu2HAmDxoChB7AheKNvCVpD4PHJwuDGn8rifMBEHmEynGHvHrf";
pub const TESTNET_SERVICE_NAME: &str = "fuel-sepolia-testnet-node";
pub const TESTNET_SYNC_HEADER_BATCH_SIZE: u32 = 100;
pub const TESTNET_RELAYER_LISTENING_CONTRACT: &str = "0x01855B78C1f8868DE70e84507ec735983bf262dA";
pub const TESTNET_RELAYER_DA_DEPLOY_HEIGHT: u32 = 5827607;
pub const TESTNET_RELAYER_LOG_PAGE_SIZE: u32 = 500;
pub const TESTNET_SYNC_BLOCK_STREAM_BUFFER_SIZE: u32 = 30;

pub const MAINNET_BOOTSTRAP_NODE: &str = "/dnsaddr/mainnet.fuel.network";
pub const MAINNET_SERVICE_NAME: &str = "fuel-mainnet-node";
pub const MAINNET_SYNC_HEADER_BATCH_SIZE: u32 = 30;
pub const MAINNET_RELAYER_LISTENING_CONTRACT: &str = "0xAEB0c00D0125A8a788956ade4f4F12Ead9f65DDf";
pub const MAINNET_RELAYER_DA_DEPLOY_HEIGHT: u32 = 20620434;
pub const MAINNET_RELAYER_LOG_PAGE_SIZE: u32 = 100;
