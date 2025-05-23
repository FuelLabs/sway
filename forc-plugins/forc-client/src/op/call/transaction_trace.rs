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

impl<'a> Node<'_> {
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

impl Display for Node<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self.receipt {
            Receipt::ScriptResult { result, gas_used } => {
                writeln!(f, "  [Script]")?;
                for child in &self.children {
                    child.fmt_with_depth(f, 0)?;
                }
                writeln!(
                    f,
                    "  [ScriptResult] result: {:?}, gas_used: {}",
                    result, gas_used
                )
            }
            _ => self.fmt_with_depth(f, 0),
        }
    }
}

impl Node<'_> {
    fn fmt_with_depth(&self, f: &mut Formatter<'_>, depth: usize) -> fmt::Result {
        let indent = if depth > 0 {
            "    │".repeat(depth)
        } else {
            "".to_string()
        };
        let prefix = "    ├─";
        let return_prefix = "    └─";

        match &self.receipt {
            Receipt::Call { to, gas, .. } => {
                writeln!(
                    f,
                    "{}{} [{}] {}",
                    indent,
                    prefix,
                    gas,
                    Color::Green.paint(format!("0x{}", to))
                )?;
                // Format children
                for child in &self.children {
                    child.fmt_with_depth(f, depth + 1)?;
                }
                Ok(())
            }
            Receipt::Return { val, .. } => {
                writeln!(f, "{}{} ← [Return] val: {}", indent, return_prefix, val)
            }
            Receipt::ReturnData { data, .. } => {
                let data_str = match data {
                    Some(data) if !data.is_empty() => format!("0x{}", hex::encode(data)),
                    _ => "()".to_string(),
                };
                writeln!(
                    f,
                    "{}{} ← {}",
                    indent,
                    return_prefix,
                    Color::BrightCyan.paint(data_str),
                )
            }
            Receipt::Revert { .. } => {
                writeln!(
                    f,
                    "{}{} ← {}",
                    indent,
                    return_prefix,
                    Color::Red.paint("[Revert]")
                )
            }
            Receipt::Panic { reason, .. } => {
                let reason_str = format!("{:?}", reason);
                writeln!(
                    f,
                    "{}{} ← {} {}",
                    indent,
                    return_prefix,
                    Color::Red.paint("[Panic]"),
                    Color::Red.paint(reason_str)
                )
            }
            Receipt::Log { rb, .. } => {
                writeln!(f, "{}{} [Log] rb: 0x{:x}", indent, prefix, rb)
            }
            Receipt::LogData { id, rb, data, .. } => {
                let data_str = match data {
                    Some(data) => {
                        let hex_str = format!("0x{}", hex::encode(data));
                        match self.abis.and_then(|abis| abis.get(id)) {
                            Some(abi) => {
                                let program_abi = sway_core::asm_generation::ProgramABI::Fuel(
                                    abi.program.clone(),
                                );
                                forc_util::tx_utils::decode_log_data(
                                    &rb.to_string(),
                                    data,
                                    &program_abi,
                                )
                                .ok()
                                .map(|decoded| decoded.value)
                                .unwrap_or(hex_str)
                            }
                            None => hex_str,
                        }
                    }
                    None => "".to_string(),
                };
                writeln!(
                    f,
                    "{}{} emit {}",
                    indent,
                    prefix,
                    Color::BrightCyan.paint(data_str),
                )
            }
            Receipt::Mint { val, .. } => {
                writeln!(f, "{}{} [Mint] val: {}", indent, prefix, val)
            }
            Receipt::Burn { val, .. } => {
                writeln!(f, "{}{} [Burn] val: {}", indent, prefix, val)
            }
            Receipt::Transfer { amount, .. } => {
                writeln!(f, "{}{} [Transfer] amount: {}", indent, prefix, amount)
            }
            Receipt::TransferOut { amount, .. } => {
                writeln!(f, "{}{} [TransferOut] amount: {}", indent, prefix, amount)
            }
            Receipt::MessageOut { amount, .. } => {
                writeln!(f, "{}{} [MessageOut] amount: {}", indent, prefix, amount)
            }
            Receipt::ScriptResult { .. } => {
                // This case is handled directly in the Display implementation
                // for the root node, and shouldn't appear inside the tree
                Ok(())
            }
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
    let trace_tree = Node::try_from_with_abis(receipts, abis)?;

    writeln!(writer, "Traces:")?;
    write!(writer, "{}", trace_tree)?;
    writeln!(writer)?;

    match trace_tree.receipt {
        Receipt::ScriptResult { result, .. } => match result {
            fuel_tx::ScriptExecutionResult::Success => writeln!(
                writer,
                "{}",
                Color::Green.paint("Transaction successfully executed.")
            )?,
            _ => writeln!(writer, "{}", Color::Red.paint("Transaction failed."))?,
        },
        _ => anyhow::bail!("Transaction trace is not a ScriptResult"),
    }
    writeln!(writer, "Gas used: {}", total_gas)?;

    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use fuels_core::types::AssetId;
    use std::str::FromStr;

    // Compare the results, ignoring whitespace differences and colors
    fn normalize(s: &str) -> String {
        // Remove ANSI color codes
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        let s = re.replace_all(s, "");
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn test_format_transaction_trace() {
        // Create sample receipts
        let contract1_id = ContractId::from_str(
            "4211b7b7a0c3104e6b9450b7a9e1b7f61912c57c3b319a956d5d7f95b480eb8e",
        )
        .unwrap();
        let contract2_id = ContractId::from_str(
            "f6035b8ac5ad76c228784d03fbba08545820715e811f574ff77300eab5e1aee9",
        )
        .unwrap();
        let asset_id =
            AssetId::from_str("f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07")
                .unwrap();

        let receipts = vec![
            Receipt::Call {
                id: ContractId::zeroed(),
                to: contract1_id,
                amount: 0,
                asset_id,
                gas: 46590,
                param1: 10480,
                param2: 10508,
                pc: 11928,
                is: 11928,
            },
            Receipt::Call {
                id: contract1_id,
                to: contract2_id,
                amount: 0,
                asset_id: AssetId::zeroed(),
                gas: 34124,
                param1: 67107840,
                param2: 67106816,
                pc: 17184,
                is: 17184,
            },
            Receipt::LogData {
                id: contract2_id,
                ra: 0,
                rb: 1515152261580153489,
                ptr: 67104256,
                len: 8,
                digest: fuel_tx::Bytes32::from([
                    0xcd, 0x26, 0x62, 0x15, 0x4e, 0x6d, 0x76, 0xb2, 0xb2, 0xb9, 0x2e, 0x70, 0xc0,
                    0xca, 0xc3, 0xcc, 0xf5, 0x34, 0xf9, 0xb7, 0x4e, 0xb5, 0xb8, 0x98, 0x19, 0xec,
                    0x50, 0x90, 0x83, 0xd0, 0x0a, 0x50,
                ]),
                pc: 20608,
                is: 17184,
                data: Some(vec![0, 0, 0, 0, 0, 0, 0, 1]),
            },
            Receipt::Revert {
                id: contract2_id,
                ra: 0,
                pc: 20612,
                is: 17184,
            },
            Receipt::ScriptResult {
                result: fuel_tx::ScriptExecutionResult::Revert,
                gas_used: 37531,
            },
        ];

        // Format the transaction trace
        let mut output = Vec::new();
        format_transaction_trace(0, &receipts, None, &mut output).unwrap();
        let trace_output = String::from_utf8(output).unwrap();

        // Expected output
        let expected_output = r#"
        Traces:
          [Script]
            ├─ [46590] 0x4211b7b7a0c3104e6b9450b7a9e1b7f61912c57c3b319a956d5d7f95b480eb8e
            │    ├─ [34124] 0xf6035b8ac5ad76c228784d03fbba08545820715e811f574ff77300eab5e1aee9
            │    │    ├─ emit 0x0000000000000001
            │    │    └─ ← [Revert]
          [ScriptResult] result: Revert, gas_used: 37531

        Transaction failed.
        Gas used: 0
        "#;

        // Compare the results, ignoring whitespace differences
        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_format_transaction_trace_simple_call() {
        let receipts_json = r#"[
            {
                "Call": {
                    "amount": 0,
                    "asset_id": "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
                    "gas": 8793,
                    "id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "is": 11680,
                    "param1": 10480,
                    "param2": 10497,
                    "pc": 11680,
                    "to": "2af09151f8276611ba65f14650970657bc42c1503d6502ffbb4d085ec37065dd"
                }
            },
            {
                "ReturnData": {
                    "data": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
                    "digest": "af5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc",
                    "id": "2af09151f8276611ba65f14650970657bc42c1503d6502ffbb4d085ec37065dd",
                    "is": 11680,
                    "len": 8,
                    "pc": 12400,
                    "ptr": 67107584
                }
            },
            {
                "Return": {
                    "id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "is": 10368,
                    "pc": 10388,
                    "val": 1
                }
            },
            {
                "ScriptResult": {
                    "gas_used": 12400,
                    "result": "Success"
                }
            }
        ]"#;

        // Get trace output from receipts JSON
        let receipts: Vec<Receipt> = serde_json::from_str(receipts_json).unwrap();
        let mut buffer = Vec::new();
        format_transaction_trace(0, &receipts, None, &mut buffer).unwrap();
        let trace_output = String::from_utf8(buffer).unwrap();

        // Expected output
        let expected_output = r#"
        Traces:
          [Script]
            ├─ [8793] 0x2af09151f8276611ba65f14650970657bc42c1503d6502ffbb4d085ec37065dd
            │    └─ ← 0x00000000000000000000000000000001
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 12400

        Transaction successfully executed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_format_transaction_trace_simple_call_log() {
        let receipts_json = r#"[
            {
                "Call": {
                    "amount": 0,
                    "asset_id": "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
                    "gas": 28311,
                    "id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "is": 11680,
                    "param1": 10480,
                    "param2": 10503,
                    "pc": 11680,
                    "to": "4a89a8fb150bf814a6610e1172baef6c68e4e273fce379fa9b30c75f584a697e"
                }
            },
            {
                "LogData": {
                    "data": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
                    "digest": "cd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50",
                    "id": "4a89a8fb150bf814a6610e1172baef6c68e4e273fce379fa9b30c75f584a697e",
                    "is": 11680,
                    "len": 8,
                    "pc": 15108,
                    "ptr": 67107328,
                    "ra": 0,
                    "rb": 1515152261580153489
                }
            },
            {
                "ReturnData": {
                    "data": [],
                    "digest": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    "id": "4a89a8fb150bf814a6610e1172baef6c68e4e273fce379fa9b30c75f584a697e",
                    "is": 11680,
                    "len": 0,
                    "pc": 12236,
                    "ptr": 0
                }
            },
            {
                "Return": {
                    "id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "is": 10368,
                    "pc": 10388,
                    "val": 1
                }
            },
            {
                "ScriptResult": {
                    "gas_used": 25412,
                    "result": "Success"
                }
            }
        ]"#;

        // Get trace output from receipts JSON
        let receipts: Vec<Receipt> = serde_json::from_str(receipts_json).unwrap();
        let mut buffer = Vec::new();
        format_transaction_trace(0, &receipts, None, &mut buffer).unwrap();
        let trace_output = String::from_utf8(buffer).unwrap();

        // Expected output
        let expected_output = r#"
        Traces:
          [Script]
            ├─ [28311] 0x4a89a8fb150bf814a6610e1172baef6c68e4e273fce379fa9b30c75f584a697e
            │     ├─ emit 0x00000000000000000000000000000001
            │     └─ ← ()
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 25412

        Transaction successfully executed.
        Gas used: 0
        "#;

        // Compare the results, ignoring whitespace differences
        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_format_transaction_trace_call_mint_transfer_burn() {
        let receipts_json = r#"[
            {
                "Call": {
                    "amount": 100,
                    "asset_id": "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
                    "gas": 46023,
                    "id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "is": 19704,
                    "param1": 10480,
                    "param2": 10496,
                    "pc": 19704,
                    "to": "5598f77f568631ad7e37e1d88b248d0c5002705ae4582fd544c9a87662a6af03"
                }
            },
            {
                "Mint": {
                    "contract_id": "5598f77f568631ad7e37e1d88b248d0c5002705ae4582fd544c9a87662a6af03",
                    "is": 19704,
                    "pc": 23288,
                    "sub_id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "val": 100
                }
            },
            {
                "TransferOut": {
                    "amount": 100,
                    "asset_id": "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
                    "id": "5598f77f568631ad7e37e1d88b248d0c5002705ae4582fd544c9a87662a6af03",
                    "is": 19704,
                    "pc": 24488,
                    "to": "de97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c"
                }
            },
            {
                "Burn": {
                    "contract_id": "5598f77f568631ad7e37e1d88b248d0c5002705ae4582fd544c9a87662a6af03",
                    "is": 19704,
                    "pc": 24512,
                    "sub_id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "val": 100
                }
            },
            {
                "ReturnData": {
                    "data": [],
                    "digest": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    "id": "5598f77f568631ad7e37e1d88b248d0c5002705ae4582fd544c9a87662a6af03",
                    "is": 19704,
                    "len": 0,
                    "pc": 20352,
                    "ptr": 0
                }
            },
            {
                "Return": {
                    "id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "is": 10368,
                    "pc": 10388,
                    "val": 1
                }
            },
            {
                "ScriptResult": {
                    "gas_used": 37228,
                    "result": "Success"
                }
            }
        ]"#;

        // Get trace output from receipts JSON
        let receipts: Vec<Receipt> = serde_json::from_str(receipts_json).unwrap();
        let mut buffer = Vec::new();
        format_transaction_trace(0, &receipts, None, &mut buffer).unwrap();
        let trace_output = String::from_utf8(buffer).unwrap();

        // Expected output
        let expected_output = r#"
        Traces:
          [Script]
            ├─ [46023] 0x5598f77f568631ad7e37e1d88b248d0c5002705ae4582fd544c9a87662a6af03
            │    ├─ [Mint] val: 100
            │    ├─ [TransferOut] amount: 100
            │    ├─ [Burn] val: 100
            │    └─ ← ()
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 37228

        Transaction successfully executed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_format_transaction_trace_nested_call_log_success() {
        let receipts_json = r#"[
            {
                "Call": {
                    "amount": 0,
                    "asset_id": "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
                    "gas": 47382,
                    "id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "is": 11928,
                    "param1": 10480,
                    "param2": 10508,
                    "pc": 11928,
                    "to": "7c05fa2efa56c4bba646af0c48db02cba34e54149785cc692ae7e297f031b12e"
                }
            },
            {
                "Call": {
                    "amount": 0,
                    "asset_id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "gas": 34914,
                    "id": "7c05fa2efa56c4bba646af0c48db02cba34e54149785cc692ae7e297f031b12e",
                    "is": 17184,
                    "param1": 67107840,
                    "param2": 67106816,
                    "pc": 17184,
                    "to": "7ecdf7b507b33131cac7295af2156fd98cd299c3512ec8c2733f920d5f8e4506"
                }
            },
            {
                "LogData": {
                    "data": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
                    "digest": "cd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50",
                    "id": "7ecdf7b507b33131cac7295af2156fd98cd299c3512ec8c2733f920d5f8e4506",
                    "is": 17184,
                    "len": 8,
                    "pc": 20616,
                    "ptr": 67104256,
                    "ra": 0,
                    "rb": 1515152261580153489
                }
            },
            {
                "ReturnData": {
                    "data": [],
                    "digest": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    "id": "7ecdf7b507b33131cac7295af2156fd98cd299c3512ec8c2733f920d5f8e4506",
                    "is": 17184,
                    "len": 0,
                    "pc": 17740,
                    "ptr": 0
                }
            },
            {
                "ReturnData": {
                    "data": [],
                    "digest": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    "id": "7c05fa2efa56c4bba646af0c48db02cba34e54149785cc692ae7e297f031b12e",
                    "is": 11928,
                    "len": 0,
                    "pc": 13332,
                    "ptr": 0
                }
            },
            {
                "Return": {
                    "id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "is": 10368,
                    "pc": 10388,
                    "val": 1
                }
            },
            {
                "ScriptResult": {
                    "gas_used": 38059,
                    "result": "Success"
                }
            }
        ]"#;

        // Get trace output from receipts JSON
        let receipts: Vec<Receipt> = serde_json::from_str(receipts_json).unwrap();
        let mut buffer = Vec::new();
        format_transaction_trace(0, &receipts, None, &mut buffer).unwrap();
        let trace_output = String::from_utf8(buffer).unwrap();

        // Expected output
        let expected_output = r#"
        Traces:
          [Script]
            ├─ [47382] 0x7c05fa2efa56c4bba646af0c48db02cba34e54149785cc692ae7e297f031b12e
            │    ├─ [34914] 0x7ecdf7b507b33131cac7295af2156fd98cd299c3512ec8c2733f920d5f8e4506
            │    │    ├─ emit 0x00000000000000000000000000000001
            │    │    └─ ← ()
            │    └─ ← ()
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 38059

        Transaction successfully executed.
        Gas used: 0
        "#;

        // Compare the results, ignoring whitespace differences
        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_format_transaction_trace_nested_call_log_success_with_multiple_calls() {
        let receipts_json = r#"[
            {
                "Call": {
                  "amount": 0,
                  "asset_id": "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
                  "gas": 105141,
                  "id": "0000000000000000000000000000000000000000000000000000000000000000",
                  "is": 11928,
                  "param1": 10480,
                  "param2": 10508,
                  "pc": 11928,
                  "to": "41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc"
                }
            },
            {
                "Call": {
                  "amount": 0,
                  "asset_id": "0000000000000000000000000000000000000000000000000000000000000000",
                  "gas": 92530,
                  "id": "41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc",
                  "is": 18608,
                  "param1": 67107840,
                  "param2": 67106816,
                  "pc": 18608,
                  "to": "38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c"
                }
            },
            {
                "ReturnData": {
                  "data": [],
                  "digest": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                  "id": "38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c",
                  "is": 18608,
                  "len": 0,
                  "pc": 20548,
                  "ptr": 0
                }
            },
            {
                "LogData": {
                  "data": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
                  "digest": "392232ec5cf9a0ef3c155ad19684907344847572e913a7a374d703fb9c9d8b5d",
                  "id": "41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc",
                  "is": 11928,
                  "len": 25,
                  "pc": 15708,
                  "ptr": 67104256,
                  "ra": 0,
                  "rb": 10098701174489624218
                }
            },
            {
                "Call": {
                  "amount": 0,
                  "asset_id": "0000000000000000000000000000000000000000000000000000000000000000",
                  "gas": 67314,
                  "id": "41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc",
                  "is": 18616,
                  "param1": 67103232,
                  "param2": 67102208,
                  "pc": 18616,
                  "to": "38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c"
                }
            },
            {
                "ReturnData": {
                  "data": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
                  "digest": "cd04a4754498e06db5a13c5f371f1f04ff6d2470f24aa9bd886540e5dce77f70",
                  "id": "38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c",
                  "is": 18616,
                  "len": 8,
                  "pc": 20896,
                  "ptr": 67099904
                }
            },
            {
                "LogData": {
                  "data": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
                  "digest": "cd04a4754498e06db5a13c5f371f1f04ff6d2470f24aa9bd886540e5dce77f70",
                  "id": "41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc",
                  "is": 11928,
                  "len": 8,
                  "pc": 15880,
                  "ptr": 67098880,
                  "ra": 0,
                  "rb": 1515152261580153489
                }
            },
            {
                "LogData": {
                  "data": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3],
                  "digest": "a3d2743e2a3ab241ba31ffc7133a43daabe6a8e624c7edc92410068a3896c871",
                  "id": "41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc",
                  "is": 11928,
                  "len": 12,
                  "pc": 15976,
                  "ptr": 67097856,
                  "ra": 0,
                  "rb": 10098701174489624218
                }
            },
            {
                "Call": {
                  "amount": 0,
                  "asset_id": "0000000000000000000000000000000000000000000000000000000000000000",
                  "gas": 53729,
                  "id": "41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc",
                  "is": 18608,
                  "param1": 67096832,
                  "param2": 67095808,
                  "pc": 18608,
                  "to": "38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c"
                }
            },
            {
                "ReturnData": {
                  "data": [],
                  "digest": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                  "id": "38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c",
                  "is": 18608,
                  "len": 0,
                  "pc": 20548,
                  "ptr": 0
                }
            },
            {
                "ReturnData": {
                  "data": [],
                  "digest": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                  "id": "41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc",
                  "is": 11928,
                  "len": 0,
                  "pc": 12580,
                  "ptr": 0
                }
            },
            {
                "Return": {
                  "id": "0000000000000000000000000000000000000000000000000000000000000000",
                  "is": 10368,
                  "pc": 10388,
                  "val": 1
                }
            },
            {
                "ScriptResult": {
                  "gas_used": 76612,
                  "result": "Success"
              }
            }
        ]"#;

        // Get trace output from receipts JSON
        let receipts: Vec<Receipt> = serde_json::from_str(receipts_json).unwrap();
        let mut buffer = Vec::new();
        format_transaction_trace(0, &receipts, None, &mut buffer).unwrap();
        let trace_output = String::from_utf8(buffer).unwrap();

        // Expected output
        let expected_output = r#"
        Traces:
          [Script]
            ├─ [105141] 0x41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc
            │    ├─ [92530] 0x38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c
            │    │    └─ ← ()
            │    ├─ emit 0x00000000000000000000000000000001
            │    ├─ [67314] 0x38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c
            │    │    └─ ← 0x00000000000000000000000000000002
            │    ├─ emit 0x00000000000000000000000000000002
            │    ├─ emit 0x00000000000000000000000000000003
            │    ├─ [53729] 0x38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c
            │    │    └─ ← ()
            │    └─ ← ()
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 76612

        Transaction successfully executed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_format_transaction_trace_nested_call_log_revert() {
        let receipts_json = r#"[
            {
                "Call": {
                    "amount": 0,
                    "asset_id": "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
                    "gas": 46590,
                    "id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "is": 11928,
                    "param1": 10480,
                    "param2": 10508,
                    "pc": 11928,
                    "to": "9a7195648cc46c832e490e9bc15ed929fa82801cc0316d1c8e0965bb5e0260a3"
                }
            },
            {
                "Call": {
                    "amount": 0,
                    "asset_id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "gas": 34124,
                    "id": "9a7195648cc46c832e490e9bc15ed929fa82801cc0316d1c8e0965bb5e0260a3",
                    "is": 17184,
                    "param1": 67107840,
                    "param2": 67106816,
                    "pc": 17184,
                    "to": "b56b9921112e2fed854ac85357a4914dab561eed98fed0cbe35c1871971dc129"
                }
            },
            {
                "LogData": {
                    "data": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
                    "digest": "cd2662154e6d76b2b2b92e70c0cac3ccf534f9b74eb5b89819ec509083d00a50",
                    "id": "b56b9921112e2fed854ac85357a4914dab561eed98fed0cbe35c1871971dc129",
                    "is": 17184,
                    "len": 8,
                    "pc": 20612,
                    "ptr": 67104256,
                    "ra": 0,
                    "rb": 1515152261580153489
                }
            },
            {
                "Revert": {
                    "id": "b56b9921112e2fed854ac85357a4914dab561eed98fed0cbe35c1871971dc129",
                    "is": 17184,
                    "pc": 20616,
                    "ra": 0
                }
            },
            {
                "ScriptResult": {
                    "gas_used": 37531,
                    "result": "Revert"
                }
            }
        ]"#;

        // Get trace output from receipts JSON
        let receipts: Vec<Receipt> = serde_json::from_str(receipts_json).unwrap();
        let mut buffer = Vec::new();
        format_transaction_trace(0, &receipts, None, &mut buffer).unwrap();
        let trace_output = String::from_utf8(buffer).unwrap();

        // Expected output - validate spaces and colours
        let expected_output = "Traces:
  [Script]
    ├─ [46590] \u{1b}[32m0x9a7195648cc46c832e490e9bc15ed929fa82801cc0316d1c8e0965bb5e0260a3\u{1b}[0m
    │    ├─ [34124] \u{1b}[32m0xb56b9921112e2fed854ac85357a4914dab561eed98fed0cbe35c1871971dc129\u{1b}[0m
    │    │    ├─ emit \u{1b}[96m0x00000000000000000000000000000001\u{1b}[0m
    │    │    └─ ← \u{1b}[31m[Revert]\u{1b}[0m
  [ScriptResult] result: Revert, gas_used: 37531

\u{1b}[31mTransaction failed.\u{1b}[0m
Gas used: 0
";
        assert_eq!(
            trace_output, expected_output,
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output, trace_output
        );
    }

    #[test]
    fn test_format_transaction_trace_nested_call_log_panic() {
        let receipts_json = r#"[
            {
                "Call": {
                    "amount": 0,
                    "asset_id": "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
                    "gas": 25156,
                    "id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "is": 11928,
                    "param1": 10480,
                    "param2": 10507,
                    "pc": 11928,
                    "to": "b09d73495f6c211ff3586a0542d5fe5fbd45a80e1cd2c1a9a787d6865cc65984"
                }
            },
            {
                "Call": {
                    "amount": 0,
                    "asset_id": "0000000000000000000000000000000000000000000000000000000000000000",
                    "gas": 12432,
                    "id": "b09d73495f6c211ff3586a0542d5fe5fbd45a80e1cd2c1a9a787d6865cc65984",
                    "is": 17184,
                    "param1": 67107840,
                    "param2": 67106816,
                    "pc": 17184,
                    "to": "75c5015d5243cfd798a7f46eb8cf3338e05197e0a271b43c4703764c82d60080"
                }
            },
            {
                "Panic": {
                    "contract_id": null,
                    "id": "75c5015d5243cfd798a7f46eb8cf3338e05197e0a271b43c4703764c82d60080",
                    "is": 17184,
                    "pc": 20260,
                    "reason": {
                        "instruction": 676152640,
                        "reason": "MemoryOwnership"
                    }
                }
            },
            {
                "ScriptResult": {
                    "gas_used": 23242,
                    "result": "Panic"
                }
            }
        ]"#;

        // Get trace output from receipts JSON
        let receipts: Vec<Receipt> = serde_json::from_str(receipts_json).unwrap();
        let mut buffer = Vec::new();
        format_transaction_trace(0, &receipts, None, &mut buffer).unwrap();
        let trace_output = String::from_utf8(buffer).unwrap();

        // Expected output
        let expected_output = "Traces:
  [Script]
    ├─ [25156] \u{1b}[32m0xb09d73495f6c211ff3586a0542d5fe5fbd45a80e1cd2c1a9a787d6865cc65984\u{1b}[0m
    │    ├─ [12432] \u{1b}[32m0x75c5015d5243cfd798a7f46eb8cf3338e05197e0a271b43c4703764c82d60080\u{1b}[0m
    │    │    └─ ← \u{1b}[31m[Panic]\u{1b}[0m \u{1b}[31mPanicInstruction { reason: MemoryOwnership, instruction: MCP { dst_addr: 0x13, src_addr: 0x14, len: 0x15 } (bytes: 28 4d 45 40) }\u{1b}[0m
  [ScriptResult] result: Panic, gas_used: 23242

\u{1b}[31mTransaction failed.\u{1b}[0m
Gas used: 0
";

        // Compare the results, ignoring whitespace differences
        let normalize = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }
}
