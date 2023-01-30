//! A `forc` plugin for submitting transactions to a Fuel network.

use fuel_gql_client::{client::types::TransactionStatus, fuel_tx};
use std::path::PathBuf;

/// The default Fuel node URL to which the transaction is submitted.
pub const DEFAULT_URL: &str = "http://127.0.0.1:4000";

/// Submit a transaction to the specified fuel node.
#[derive(Debug, clap::Parser)]
#[clap(about, version)]
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
#[derive(Debug, clap::Args)]
pub struct Network {
    /// The URL of the Fuel node to which we're submitting the transaction.
    #[clap(long, default_value_t = String::from(DEFAULT_URL))]
    pub node_url: String,
    /// Whether or not to await confirmation that the transaction has been committed.
    ///
    /// When `true`, await commitment and output the transaction status.
    /// When `false`, do not await confirmation and simply output the transaction ID.
    #[clap(long = "await", default_value_t = true)]
    pub await_: bool,
}

/// Options related to the transaction status.
#[derive(Debug, clap::Args)]
pub struct TxStatus {
    /// Output the resulting transaction status as JSON rather than the default output.
    #[clap(long = "tx-status-json", default_value_t = false)]
    pub json: bool,
}

/// Deserialize a `Transaction` from the given file into memory.
pub fn read_tx(path: &std::path::Path) -> anyhow::Result<fuel_tx::Transaction> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let tx: fuel_tx::Transaction = if path.ends_with("json") {
        serde_json::from_reader(reader)?
    } else {
        let tx_bytes = std::fs::read(path)?;
        let (_bytes, tx) = fuel_tx::Transaction::try_from_bytes(&tx_bytes)?;
        tx
    };
    Ok(tx)
}

/// Format the transaction status in a more human-friendly manner.
pub fn fmt_status(status: &TransactionStatus, s: &mut String) -> anyhow::Result<()> {
    use std::fmt::Write;
    match status {
        TransactionStatus::Submitted { submitted_at } => {
            writeln!(s, "Transaction Submitted at {:?}", submitted_at.0)?;
        }
        TransactionStatus::Success {
            block_id,
            time,
            program_state,
        } => {
            writeln!(s, "Transaction Succeeded")?;
            writeln!(s, "  Block ID:      {block_id}")?;
            writeln!(s, "  Time:          {time:?}")?;
            writeln!(s, "  Program State: {program_state:?}")?;
        }
        TransactionStatus::SqueezedOut { reason } => {
            writeln!(s, "Transaction Squeezed Out: {reason}")?;
        }
        TransactionStatus::Failure {
            block_id,
            time,
            reason,
            program_state,
        } => {
            writeln!(s, "Transaction Failed")?;
            writeln!(s, "  Reason: {reason}")?;
            writeln!(s, "  Block ID:      {block_id}")?;
            writeln!(s, "  Time:          {time:?}")?;
            writeln!(s, "  Program State: {program_state:?}")?;
        }
    }
    Ok(())
}

/// Print the status to stdout.
pub fn print_status(status: &TransactionStatus) {
    let mut string = String::new();
    fmt_status(status, &mut string).expect("formatting to `String` is infallible");
    println!("{string}");
}

/// Print the status to stdout in its JSON representation.
pub fn print_status_json(status: &TransactionStatus) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(status)?;
    println!("{json}");
    Ok(())
}
