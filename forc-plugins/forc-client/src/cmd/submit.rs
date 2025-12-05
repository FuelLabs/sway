use crate::NodeTarget;
use devault::Devault;
use std::path::PathBuf;

forc_types::cli_examples! {
    super::Command {
        [ Submit a transaction from a json file => "forc submit {path}/mint.json" ]
        [ Submit a transaction from a json file and wait for confirmation => "forc submit {path}/mint.json --await true" ]
        [ Submit a transaction from a json file and get output in json => "forc submit {path}/mint.json --tx-status-json true" ]
        [ Submit a transaction from a json file to testnet => "forc submit {path}/mint.json --testnet" ]
        [ Submit a transaction from a json file to a local net => "forc submit {path}/mint.json --target local" ]
    }
}

/// Submit a transaction to the specified fuel node.
#[derive(Debug, Default, clap::Parser)]
#[clap(about, version, after_help = help())]
pub struct Command {
    #[clap(flatten)]
    pub network: Network,
    #[clap(flatten)]
    pub tx_status: TxStatus,
    /// Path to the Transaction that is to be submitted to the Fuel node.
    ///
    /// Paths to files ending with `.json` will be deserialized from JSON.
    /// Paths to files ending with `.bin` will be deserialized from bytes
    /// using the `fuel_tx::Transaction::try_from_bytes` constructor.
    pub tx_path: PathBuf,
}

/// Options related to networking.
#[derive(Debug, Devault, clap::Args)]
pub struct Network {
    #[clap(flatten)]
    pub node: NodeTarget,
    /// Whether or not to await confirmation that the transaction has been committed.
    ///
    /// When `true`, await commitment and output the transaction status.
    /// When `false`, do not await confirmation and simply output the transaction ID.
    #[clap(long = "await", default_value = "true", action(clap::ArgAction::Set))]
    #[devault("true")]
    pub await_: bool,
}

/// Options related to the transaction status.
#[derive(Debug, Default, clap::Args)]
pub struct TxStatus {
    /// Output the resulting transaction status as JSON rather than the default output.
    #[clap(
        long = "tx-status-json",
        default_value = "false",
        action(clap::ArgAction::Set)
    )]
    pub json: bool,
}
