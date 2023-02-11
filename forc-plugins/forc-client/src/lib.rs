pub mod cmd;
pub mod op;
mod util;

pub mod default {
    /// Default to localhost to favour the common case of testing.
    pub const NODE_URL: &str = sway_utils::constants::DEFAULT_NODE_URL;
}
