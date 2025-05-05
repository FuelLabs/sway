use anyhow::Result;
use clap::Args;
use fuel_abi_types::error_codes::ErrorSignal;
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

pub struct RevertPosition {
    pub pkg: String,
    pub file: String,
    pub line: u64,
    pub column: u64,
}

// TODO: Move `RevertInfo` and related types to `fuel-abi-types` crate.
//       We temporarily keep it here to get the support for `panic` expression in `forc test` ASAP,
//       without waiting for the next `fuel-abi-types` release.

/// Information about a revert that occurred during a transaction execution.
pub struct RevertInfo {
    pub revert_code: u64,
    pub kind: RevertKind,
}

pub enum RevertKind {
    /// This is the most general kind of a revert, where we only know the revert code.
    /// E.g., reverts caused by `__revert` calls.
    RawRevert,
    /// Reverts caused by known functions, like, e.g., `assert` or `require`, that provide known error signals.
    /// For such reverts, we can provide the error message.
    KnownErrorSignal { err_msg: String },
    Panic {
        err_msg: Option<String>,
        err_val: Option<String>,
        pos: RevertPosition,
    },
}

impl RevertInfo {
    pub fn raw_revert(revert_code: u64) -> Self {
        Self {
            revert_code,
            kind: RevertKind::RawRevert,
        }
    }

    pub fn new(
        revert_code: u64,
        program_abi: Option<&fuel_abi_types::abi::program::ProgramABI>,
        logs: &[fuel_tx::Receipt],
    ) -> Self {
        /// Types that implement the `std::marker::Error` trait, and whose instances
        /// can be used as arguments to the `panic` expression.
        enum ErrorType {
            Unknown,
            Unit,
            Str,
            Enum,
        }

        impl ErrorType {
            fn from_type_name(type_name: &str) -> Self {
                match type_name {
                    "()" => ErrorType::Unit,
                    "str" => ErrorType::Str,
                    name if name.starts_with("enum ") => ErrorType::Enum,
                    _ => ErrorType::Unknown,
                }
            }
        }

        if let Ok(error_signal) = ErrorSignal::try_from_revert_code(revert_code) {
            Self {
                revert_code,
                kind: RevertKind::KnownErrorSignal {
                    err_msg: error_signal.to_string(),
                },
            }
        } else if let Some(program_abi) = program_abi {
            // We have the program ABI available, and can try to extract more information about the revert.
            if let Some(error_details) = program_abi
                .error_codes
                .as_ref()
                .and_then(|error_codes| error_codes.get(&revert_code))
            {
                // If we have an ABI error code, we always know the position.
                let pos = RevertPosition {
                    pkg: error_details.pos.pkg.clone(),
                    file: error_details.pos.file.clone(),
                    line: error_details.pos.line,
                    column: error_details.pos.column,
                };

                // Message and log ID are mutually exclusive.
                let (err_msg, err_val) = if let Some(msg) = &error_details.msg {
                    (Some(msg.clone()), None)
                } else if let Some(log_id) = &error_details.log_id {
                    // Because we got the error code, we know that the revert is a result of `panic`king.
                    // The log receipt created by the `panic` expression will be the last one in the logs.
                    let err_val = logs
                        .last()
                        .and_then(|log| {
                            if let fuel_tx::Receipt::LogData {
                                data: Some(data), ..
                            } = log
                            {
                                decode_fuel_vm_log_data(log_id, data, program_abi).ok()
                            } else {
                                None
                            }
                        })
                        .map(|decoded_log| decoded_log.value);

                    match program_abi
                        .logged_types
                        .as_ref()
                        .unwrap_or(&vec![])
                        .iter()
                        .find(|logged_type| logged_type.log_id == *log_id)
                        .and_then(|logged_type| {
                            program_abi.concrete_types.iter().find(|concrete_type| {
                                concrete_type.concrete_type_id == logged_type.concrete_type_id
                            })
                        })
                        .map(|type_decl| &type_decl.type_field)
                    {
                        // All of the `(None, err_val)` cases below can happen only if the ABI is malformed.
                        // We handle that case gracefully by returning `None` for the error message,
                        // but still returning the error value if it is provided.
                        // Note that not having an error value is also possible only if the ABI is malformed.
                        Some(error_type_name) => match ErrorType::from_type_name(error_type_name) {
                            ErrorType::Unit => (None, err_val),
                            ErrorType::Str => {
                                // This is the case where the error value is a non-const evaluated string slice.
                                // The error message will be null in the JSON ABI and the log value will be the string slice
                                // decoded like: `AsciiString { data: "<the actual error message>" }`.
                                // In this case, we will actually show `<the actual error message>` as the error message
                                // and set the error value to `None`.
                                // The `AsciiString { data: "<the actual error message>" }` will still be displayed in the logs,
                                // We "parse" the error message out, by gracefully extracting it from the decoded logged value.
                                if let Some(err_val) = err_val {
                                    let left_quote_index = err_val.find('"').unwrap_or_default();
                                    let right_quote_index = err_val.rfind('"').unwrap_or_default();
                                    if left_quote_index < right_quote_index {
                                        let err_msg = err_val[left_quote_index..right_quote_index]
                                            .trim_matches('"');
                                        (Some(err_msg.to_string()), None)
                                    } else {
                                        (None, Some(err_val)) // Malformed error value, handle gracefully.
                                    }
                                } else {
                                    (None, err_val) // Malformed ABI, handle gracefully.
                                }
                            }
                            ErrorType::Enum => {
                                if let Some(err_val) = err_val {
                                    let err_msg = program_abi
                                        .metadata_types
                                        .iter()
                                        .find(|metadata_type| {
                                            metadata_type.type_field == *error_type_name
                                        })
                                        .and_then(|metadata_type| {
                                            metadata_type.components.as_ref().and_then(
                                                |components| {
                                                    // The component name will be the name of the error enum variant.
                                                    // We extract the concrete error enum variant name from the logged error value.
                                                    // The logged error value will either be a `SomeErrorVariant` or `SomeErrorVariant(value)`.
                                                    // So, the name will be the first part of the string, up to the first `(` if it exists.
                                                    // TODO: Is there a better way to match the logged error value to the component name?
                                                    err_val.split('(').next().map(|variant_name| {
                                                        components
                                                            .iter()
                                                            .find(|component| {
                                                                component.name.as_str()
                                                                    == variant_name
                                                            })
                                                            .and_then(|component| {
                                                                component.error_message.clone()
                                                            })
                                                    })
                                                },
                                            )
                                        })
                                        .flatten();

                                    (err_msg, Some(err_val))
                                } else {
                                    (None, err_val) // Malformed ABI, handle gracefully.
                                }
                            }
                            ErrorType::Unknown => (None, err_val), // Malformed ABI, handle gracefully.
                        },
                        None => (None, err_val), // Malformed ABI, handle gracefully.
                    }
                } else {
                    (None, None)
                };

                Self {
                    revert_code,
                    kind: RevertKind::Panic {
                        err_msg,
                        err_val,
                        pos,
                    },
                }
            } else {
                Self::raw_revert(revert_code)
            }
        } else {
            // No known error signal, and no ABI available. We can't extract any additional information.
            Self::raw_revert(revert_code)
        }
    }
}
