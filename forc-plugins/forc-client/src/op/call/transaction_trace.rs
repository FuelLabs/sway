use crate::op::call::Abi;
use ansiterm::Color;
use anyhow::Result;
use fuel_tx::Receipt;
use fuels_core::types::ContractId;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

/// A node in the transaction trace tree
pub(crate) struct Node<'a> {
    receipt: Receipt,
    children: Vec<Node<'a>>,
    abis: Option<&'a HashMap<ContractId, Abi>>,
}

impl<'a> Node<'a> {
    /// Create a new Node from receipts with ABI information
    pub(crate) fn try_from_with_abis<'b>(
        receipts: &[Receipt],
        abis: Option<&'b HashMap<ContractId, Abi>>,
    ) -> Result<Node<'b>, anyhow::Error> {
        // Find the script result receipt
        let script_result_receipt = receipts
            .iter()
            .find(|r| matches!(r, Receipt::ScriptResult { .. }))
            .ok_or_else(|| anyhow::anyhow!("ScriptResult receipt not found in the receipt list"))?;

        // Create a root node (script node) - ScriptResult receipt
        let mut script_node = Node {
            receipt: script_result_receipt.clone(),
            children: Vec::new(),
            abis,
        };

        // Process all receipts and build the tree
        let mut index = 0;
        while index < receipts.len() {
            let (new_index, maybe_node) =
                Node::process_receipt(receipts, index, &ContractId::zeroed(), abis);
            index = new_index;
            if let Some(node) = maybe_node {
                script_node.children.push(node);
            }
        }

        Ok(script_node)
    }

    /// Process a receipt and its children, returning the new index and an optional node
    fn process_receipt(
        receipts: &[Receipt],
        start_index: usize,
        parent_id: &ContractId,
        abis: Option<&'a HashMap<ContractId, Abi>>,
    ) -> (usize, Option<Node<'a>>) {
        if start_index >= receipts.len() {
            return (start_index, None);
        }

        let receipt = &receipts[start_index];

        // Skip ScriptResult - these are handled separately at the root level
        if matches!(receipt, Receipt::ScriptResult { .. }) {
            return (start_index + 1, None);
        }

        match receipt {
            Receipt::Call { id, to, .. } if *id == *parent_id => {
                // Create a Call node
                let mut call_node = Node {
                    receipt: receipt.clone(),
                    children: Vec::new(),
                    abis,
                };

                // Process children of this call
                let mut index = start_index + 1;
                let mut is_terminated = false;

                while index < receipts.len() && !is_terminated {
                    let receipt = &receipts[index];

                    // Check if this is another Call with the same id and to (would be a different call)
                    if let Receipt::Call {
                        id: next_id,
                        to: next_to,
                        ..
                    } = receipt
                    {
                        if *next_id == *id && *next_to == *to {
                            // Found another call to the same contract, stop processing
                            break;
                        }
                    }

                    let (new_index, maybe_child) = match receipt {
                        // If it's a nested call that belongs to this call
                        Receipt::Call { id: child_id, .. } if *child_id == *to => {
                            Node::process_receipt(receipts, index, to, abis)
                        }
                        // Otherwise, check if it's a terminal receipt or a child of this call
                        _ => {
                            let node_id = Node::get_receipt_id(receipt);
                            if node_id == *to {
                                // This receipt belongs to this call
                                let (return_index, node) = (
                                    index + 1,
                                    Node {
                                        receipt: receipt.clone(),
                                        children: Vec::new(),
                                        abis,
                                    },
                                );

                                // Check if this is a terminal receipt
                                is_terminated = matches!(
                                    receipt,
                                    Receipt::Return { .. }
                                        | Receipt::ReturnData { .. }
                                        | Receipt::Revert { .. }
                                        | Receipt::Panic { .. }
                                );

                                (return_index, Some(node))
                            } else {
                                // This receipt doesn't belong to this call
                                break;
                            }
                        }
                    };

                    index = new_index;

                    if let Some(child) = maybe_child {
                        call_node.children.push(child);
                    }
                }

                (index, Some(call_node))
            }
            _ if Node::get_receipt_id(receipt) == *parent_id => {
                // This is a direct child of the parent (not a Call)
                (
                    start_index + 1,
                    Some(Node {
                        receipt: receipt.clone(),
                        children: Vec::new(),
                        abis,
                    }),
                )
            }
            _ => {
                // This receipt doesn't belong to this parent
                (start_index + 1, None)
            }
        }
    }

    /// Extract the contract ID or equivalent from a receipt
    fn get_receipt_id(receipt: &Receipt) -> ContractId {
        match receipt {
            Receipt::Call { id, .. }
            | Receipt::Return { id, .. }
            | Receipt::ReturnData { id, .. }
            | Receipt::Panic { id, .. }
            | Receipt::Revert { id, .. }
            | Receipt::Log { id, .. }
            | Receipt::LogData { id, .. }
            | Receipt::Transfer { id, .. }
            | Receipt::TransferOut { id, .. } => *id,
            Receipt::Mint { contract_id, .. } | Receipt::Burn { contract_id, .. } => *contract_id,
            Receipt::ScriptResult { .. } | Receipt::MessageOut { .. } => ContractId::zeroed(),
        }
    }
}

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
