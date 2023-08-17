pub mod cmd;
pub mod op;
mod util;

pub mod default {
    /// Default to localhost to favour the common case of testing.
    pub const NODE_URL: &str = sway_utils::constants::DEFAULT_NODE_URL;
    pub const BETA_2_ENDPOINT_URL: &str = "node-beta-2.fuel.network/graphql";
    pub const BETA_3_ENDPOINT_URL: &str = "beta-3.fuel.network/graphql";
    pub const BETA_4_ENDPOINT_URL: &str = "beta-4.fuel.network/graphql";
    pub const BETA_4_FAUCET_URL: &str = "https://faucet-beta-4.fuel.network";
}
