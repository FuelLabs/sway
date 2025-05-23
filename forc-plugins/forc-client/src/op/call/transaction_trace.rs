use crate::op::call::Abi;
use ansiterm::Color;
use anyhow::Result;
use fuel_tx::Receipt;
use fuels_core::types::ContractId;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

/// Format transaction receipts into a hierarchical trace visualization.
/// Optionally, provide a map of contract IDs to their ABIs for function name and type lookup/resolution.
pub(crate) fn format_transaction_trace<W: std::io::Write>(
    total_gas: u64,
    receipts: &[Receipt],
    abis: Option<&HashMap<ContractId, Abi>>,
    writer: &mut W,
) -> Result<()> {
    Ok(())
}
