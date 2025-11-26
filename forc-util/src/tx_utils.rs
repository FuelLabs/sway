use anyhow::Result;
use clap::Args;
use fuel_abi_types::revert_info::RevertInfo;
use fuels_core::{codec::ABIDecoder, types::param_types::ParamType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sway_core::{asm_generation::ProgramABI, fuel_prelude::fuel_tx};

/// Added salt used to derive the contract ID.
#[derive(Debug, Args, Default, Deserialize, Serialize)]
pub struct Salt {
    /// Added salt used to derive the contract ID.
    ///
    /// By default, this is
    /// `0x0000000000000000000000000000000000000000000000000000000000000000`.
    #[clap(long = "salt")]
    pub salt: Option<fuel_tx::Salt>,
}

/// Format `Log` and `LogData` receipts.
pub fn format_log_receipts(receipts: &[fuel_tx::Receipt], pretty_print: bool) -> Result<String> {
    let mut receipt_to_json_array = serde_json::to_value(receipts)?;
    for (rec_index, receipt) in receipts.iter().enumerate() {
        let rec_value = receipt_to_json_array.get_mut(rec_index).ok_or_else(|| {
            anyhow::anyhow!(
                "Serialized receipts does not contain {} th index",
                rec_index
            )
        })?;
        match receipt {
            fuel_tx::Receipt::LogData {
                data: Some(data), ..
            } => {
                if let Some(v) = rec_value.pointer_mut("/LogData/data") {
                    *v = hex::encode(data).into();
                }
            }
            fuel_tx::Receipt::ReturnData {
                data: Some(data), ..
            } => {
                if let Some(v) = rec_value.pointer_mut("/ReturnData/data") {
                    *v = hex::encode(data).into();
                }
            }
            _ => {}
        }
    }
    if pretty_print {
        Ok(serde_json::to_string_pretty(&receipt_to_json_array)?)
    } else {
        Ok(serde_json::to_string(&receipt_to_json_array)?)
    }
}

/// A `LogData` decoded into a human readable format with its type information.
pub struct DecodedLog {
    pub value: String,
}

pub fn decode_log_data(
    log_id: &str,
    log_data: &[u8],
    program_abi: &ProgramABI,
) -> anyhow::Result<DecodedLog> {
    match program_abi {
        ProgramABI::Fuel(program_abi) => decode_fuel_vm_log_data(log_id, log_data, program_abi),
        _ => Err(anyhow::anyhow!(
            "only Fuel VM is supported for log decoding"
        )),
    }
}

pub fn decode_fuel_vm_log_data(
    log_id: &str,
    log_data: &[u8],
    program_abi: &fuel_abi_types::abi::program::ProgramABI,
) -> anyhow::Result<DecodedLog> {
    let program_abi =
        fuel_abi_types::abi::unified_program::UnifiedProgramABI::from_counterpart(program_abi)?;

    // Create type lookup (id, TypeDeclaration)
    let type_lookup = program_abi
        .types
        .iter()
        .map(|decl| (decl.type_id, decl.clone()))
        .collect::<HashMap<_, _>>();

    let logged_type_lookup: HashMap<_, _> = program_abi
        .logged_types
        .iter()
        .flatten()
        .map(|logged_type| (logged_type.log_id.as_str(), logged_type.application.clone()))
        .collect();

    let type_application = logged_type_lookup
        .get(&log_id)
        .ok_or_else(|| anyhow::anyhow!("log id is missing"))?;

    let abi_decoder = ABIDecoder::default();
    let param_type = ParamType::try_from_type_application(type_application, &type_lookup)?;
    let decoded_str = abi_decoder.decode_as_debug_str(&param_type, log_data)?;
    let decoded_log = DecodedLog { value: decoded_str };

    Ok(decoded_log)
}

/// Build [`RevertInfo`] from VM receipts and an optional program ABI.
/// This extracts the latest revert code from receipts (or a provided hint) and
/// decodes panic metadata (message/value/backtrace) using the ABI metadata if available.
pub fn revert_info_from_receipts(
    receipts: &[fuel_tx::Receipt],
    program_abi: Option<&fuel_abi_types::abi::program::ProgramABI>,
    revert_code_hint: Option<u64>,
) -> Option<RevertInfo> {
    let revert_code = receipts
        .iter()
        .rev()
        .find_map(|receipt| match receipt {
            fuel_tx::Receipt::Revert { ra, .. } => Some(*ra),
            _ => None,
        })
        .or(revert_code_hint)?;

    let decode_last_log_data =
        |log_id: &str, program_abi: &fuel_abi_types::abi::program::ProgramABI| {
            receipts.iter().rev().find_map(|receipt| match receipt {
                fuel_tx::Receipt::LogData {
                    data: Some(data), ..
                } => decode_fuel_vm_log_data(log_id, data, program_abi)
                    .ok()
                    .map(|decoded| decoded.value),
                _ => None,
            })
        };

    Some(RevertInfo::new(
        revert_code,
        program_abi,
        decode_last_log_data,
    ))
}
