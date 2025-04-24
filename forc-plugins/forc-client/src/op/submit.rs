use crate::cmd;
use anyhow::Context;
use fuel_core_client::client::{types::TransactionStatus, FuelClient};
use fuel_crypto::fuel_types::canonical::Deserialize;

/// A command for submitting transactions to a Fuel network.
pub async fn submit(cmd: cmd::Submit) -> anyhow::Result<()> {
    let tx = read_tx(&cmd.tx_path)?;
    let node_url = cmd.network.node.get_node_url(&None)?;
    let client = FuelClient::new(node_url)?;
    if cmd.network.await_ {
        let status = client
            .submit_and_await_commit(&tx)
            .await
            .context("Submission of tx or awaiting commit failed")?;
        if cmd.tx_status.json {
            print_status_json(&status)?;
        } else {
            print_status(&status);
        }
    } else {
        let id = client.submit(&tx).await.context("Failed to submit tx")?;
        println!("{id}");
    }
    Ok(())
}

/// Deserialize a `Transaction` from the given file into memory.
pub fn read_tx(path: &std::path::Path) -> anyhow::Result<fuel_tx::Transaction> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    fn has_extension(path: &std::path::Path, ext: &str) -> bool {
        path.extension().and_then(|ex| ex.to_str()) == Some(ext)
    }
    let tx: fuel_tx::Transaction = if has_extension(path, "json") {
        serde_json::from_reader(reader)?
    } else if has_extension(path, "bin") {
        let tx_bytes = std::fs::read(path)?;
        fuel_tx::Transaction::from_bytes(&tx_bytes).map_err(anyhow::Error::msg)?
    } else {
        anyhow::bail!(r#"Unsupported transaction file extension, expected ".json" or ".bin""#);
    };
    Ok(tx)
}

/// Format the transaction status in a more human-friendly manner.
pub fn fmt_status(status: &TransactionStatus, s: &mut String) -> anyhow::Result<()> {
    use chrono::TimeZone;
    use std::fmt::Write;
    match status {
        TransactionStatus::Submitted { submitted_at } => {
            writeln!(s, "Transaction Submitted at {:?}", submitted_at.0)?;
        }
        TransactionStatus::Success {
            block_height,
            time,
            program_state,
            ..
        } => {
            let utc = chrono::Utc.timestamp_nanos(time.to_unix());
            writeln!(s, "Transaction Succeeded")?;
            writeln!(s, "  Block ID:      {block_height}")?;
            writeln!(s, "  Time:          {utc}",)?;
            writeln!(s, "  Program State: {program_state:?}")?;
        }
        TransactionStatus::SqueezedOut { reason } => {
            writeln!(s, "Transaction Squeezed Out: {reason}")?;
        }
        TransactionStatus::Failure {
            block_height,
            time,
            reason,
            program_state,
            ..
        } => {
            let utc = chrono::Utc.timestamp_nanos(time.to_unix());
            writeln!(s, "Transaction Failed")?;
            writeln!(s, "  Reason: {reason}")?;
            writeln!(s, "  Block ID:      {block_height}")?;
            writeln!(s, "  Time:          {utc}")?;
            writeln!(s, "  Program State: {program_state:?}")?;
        }
        TransactionStatus::PreconfirmationSuccess {
            total_gas,
            transaction_id,
            receipts,
            ..
        } => {
            writeln!(s, "Transaction Preconfirmatino Succeeded")?;
            writeln!(s, "  Total Gas:      {total_gas}")?;
            writeln!(s, "  Transaction Id:          {transaction_id}",)?;
            writeln!(s, "  Receipts: {receipts:?}")?;
        }
        TransactionStatus::PreconfirmationFailure {
            total_gas,
            transaction_id,
            receipts,
            reason,
            ..
        } => {
            writeln!(s, "Transaction Preconfirmation Failed")?;
            writeln!(s, "  Total Gas:      {total_gas}")?;
            writeln!(s, "  Transaction Id:          {transaction_id}",)?;
            writeln!(s, "  Receipts: {receipts:?}")?;
            writeln!(s, "  Reason: {reason:?}")?;
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
